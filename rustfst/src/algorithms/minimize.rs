use std::cmp::max;
use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;
use std::marker::PhantomData;

use anyhow::Result;
use binary_heap_plus::BinaryHeap;
use stable_bst::TreeMap;

use crate::algorithms::encode::EncodeType;
use crate::algorithms::factor_weight::factor_iterators::GallicFactorLeft;
use crate::algorithms::factor_weight::{factor_weight, FactorWeightOptions, FactorWeightType};
use crate::algorithms::partition::Partition;
use crate::algorithms::queues::LifoQueue;
use crate::algorithms::tr_compares::ILabelCompare;
use crate::algorithms::tr_mappers::QuantizeMapper;
use crate::algorithms::tr_unique;
use crate::algorithms::weight_converters::{FromGallicConverter, ToGallicConverter};
use crate::algorithms::Queue;
use crate::algorithms::{
    connect,
    encode::{decode, encode},
    tr_map, tr_sort, weight_convert, ReweightType,
};
use crate::algorithms::{push_weights_with_config, reverse, PushWeightsConfig};
use crate::fst_impls::VectorFst;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{AllocableFst, CoreFst, ExpandedFst, Fst, MutableFst};
use crate::semirings::{
    GallicWeightLeft, Semiring, SemiringProperties, WeaklyDivisibleSemiring, WeightQuantize,
};
use crate::EPS_LABEL;
use crate::KDELTA;
use crate::{Label, StateId, Trs};
use crate::{Tr, KSHORTESTDELTA};
use itertools::Itertools;
use std::cell::RefCell;
use std::rc::Rc;

/// Configuration for minimization.
#[derive(Clone, Copy, PartialOrd, PartialEq)]
pub struct MinimizeConfig {
    pub delta: f32,
    pub allow_nondet: bool,
}

impl MinimizeConfig {
    pub fn new(delta: f32, allow_nondet: bool) -> Self {
        Self {
            delta,
            allow_nondet,
        }
    }

    pub fn with_delta(self, delta: f32) -> Self {
        Self { delta, ..self }
    }

    pub fn with_allow_nondet(self, allow_nondet: bool) -> Self {
        Self {
            allow_nondet,
            ..self
        }
    }
}

impl Default for MinimizeConfig {
    fn default() -> Self {
        Self {
            delta: KSHORTESTDELTA,
            allow_nondet: false,
        }
    }
}

/// In place minimization of deterministic weighted automata and transducers,
/// and also non-deterministic ones if they use an idempotent semiring.
/// For transducers, the algorithm produces a compact factorization of the minimal transducer.
pub fn minimize<W, F>(ifst: &mut F) -> Result<()>
where
    F: MutableFst<W> + ExpandedFst<W> + AllocableFst<W>,
    W: WeaklyDivisibleSemiring + WeightQuantize,
    W::ReverseWeight: WeightQuantize,
{
    minimize_with_config(ifst, MinimizeConfig::default())
}

/// In place minimization of deterministic weighted automata and transducers,
/// and also non-deterministic ones if they use an idempotent semiring.
/// For transducers, the algorithm produces a compact factorization of the minimal transducer.
pub fn minimize_with_config<W, F>(ifst: &mut F, config: MinimizeConfig) -> Result<()>
where
    F: MutableFst<W> + ExpandedFst<W> + AllocableFst<W>,
    W: WeaklyDivisibleSemiring + WeightQuantize,
    W::ReverseWeight: WeightQuantize,
{
    let delta = config.delta;
    let allow_nondet = config.allow_nondet;

    let props = ifst.compute_and_update_properties(
        FstProperties::ACCEPTOR
            | FstProperties::I_DETERMINISTIC
            | FstProperties::WEIGHTED
            | FstProperties::UNWEIGHTED,
    )?;

    let allow_acyclic_minimization = if props.contains(FstProperties::I_DETERMINISTIC) {
        true
    } else {
        if !W::properties().contains(SemiringProperties::IDEMPOTENT) {
            bail!("Cannot minimize a non-deterministic FST over a non-idempotent semiring")
        } else if !allow_nondet {
            bail!("Refusing to minimize a non-deterministic FST with allow_nondet = false")
        }

        false
    };

    if !props.contains(FstProperties::ACCEPTOR) {
        // Weighted transducer
        let mut to_gallic = ToGallicConverter {};
        let mut gfst: VectorFst<GallicWeightLeft<W>> = weight_convert(ifst, &mut to_gallic)?;
        let push_weights_config = PushWeightsConfig::default().with_delta(delta);
        push_weights_with_config(
            &mut gfst,
            ReweightType::ReweightToInitial,
            push_weights_config,
        )?;

        let quantize_mapper = QuantizeMapper::new(delta);
        tr_map(&mut gfst, &quantize_mapper)?;

        let encode_table = encode(&mut gfst, EncodeType::EncodeWeightsAndLabels)?;

        acceptor_minimize(&mut gfst, allow_acyclic_minimization)?;

        decode(&mut gfst, encode_table)?;

        let factor_opts: FactorWeightOptions = FactorWeightOptions {
            delta: KDELTA,
            mode: FactorWeightType::FACTOR_FINAL_WEIGHTS | FactorWeightType::FACTOR_ARC_WEIGHTS,
            final_ilabel: 0,
            final_olabel: 0,
            increment_final_ilabel: false,
            increment_final_olabel: false,
        };

        let fwfst: VectorFst<_> =
            factor_weight::<_, VectorFst<GallicWeightLeft<W>>, _, _, GallicFactorLeft<W>>(
                &gfst,
                factor_opts,
            )?;

        let mut from_gallic = FromGallicConverter {
            superfinal_label: EPS_LABEL,
        };
        *ifst = weight_convert(&fwfst, &mut from_gallic)?;

        Ok(())
    } else if props.contains(FstProperties::WEIGHTED) {
        // Weighted acceptor
        let push_weights_config = PushWeightsConfig::default().with_delta(delta);
        push_weights_with_config(ifst, ReweightType::ReweightToInitial, push_weights_config)?;
        let quantize_mapper = QuantizeMapper::new(delta);
        tr_map(ifst, &quantize_mapper)?;
        let encode_table = encode(ifst, EncodeType::EncodeWeightsAndLabels)?;
        acceptor_minimize(ifst, allow_acyclic_minimization)?;
        decode(ifst, encode_table)
    } else {
        // Unweighted acceptor
        acceptor_minimize(ifst, allow_acyclic_minimization)
    }
}

/// In place minimization for weighted final state acceptor.
/// If `allow_acyclic_minimization` is true and the input is acyclic, then a specific
/// minimization is applied.
///
/// An error is returned if the input fst is not a weighted acceptor.
pub fn acceptor_minimize<W: Semiring, F: MutableFst<W> + ExpandedFst<W>>(
    ifst: &mut F,
    allow_acyclic_minimization: bool,
) -> Result<()> {
    let props = ifst.compute_and_update_properties(
        FstProperties::ACCEPTOR | FstProperties::UNWEIGHTED | FstProperties::ACYCLIC,
    )?;
    if !props.contains(FstProperties::ACCEPTOR | FstProperties::UNWEIGHTED) {
        bail!("FST is not an unweighted acceptor");
    }

    connect(ifst)?;

    if ifst.num_states() == 0 {
        return Ok(());
    }

    if allow_acyclic_minimization && props.contains(FstProperties::ACYCLIC) {
        // Acyclic minimization
        tr_sort(ifst, ILabelCompare {});
        let minimizer = AcyclicMinimizer::new(ifst)?;
        merge_states(minimizer.get_partition(), ifst)?;
    } else {
        let p = cyclic_minimize(ifst)?;
        merge_states(p, ifst)?;
    }

    tr_unique(ifst);

    Ok(())
}

fn merge_states<W: Semiring, F: MutableFst<W>>(
    partition: Rc<RefCell<Partition>>,
    fst: &mut F,
) -> Result<()> {
    let mut state_map = vec![None; partition.borrow().num_classes()];

    for (i, state_map_i) in state_map
        .iter_mut()
        .enumerate()
        .take(partition.borrow().num_classes())
    {
        *state_map_i = partition.borrow().iter(i).next();
    }

    for c in 0..partition.borrow().num_classes() {
        for s in partition.borrow().iter(c) {
            if s == state_map[c].unwrap() {
                let mut it_tr = fst.tr_iter_mut(s as StateId)?;
                for idx_tr in 0..it_tr.len() {
                    let tr = unsafe { it_tr.get_unchecked(idx_tr) };
                    let nextstate =
                        state_map[partition.borrow().get_class_id(tr.nextstate as usize)].unwrap();
                    unsafe { it_tr.set_nextstate_unchecked(idx_tr, nextstate as StateId) };
                }
            } else {
                for tr in fst
                    .get_trs(s as StateId)?
                    .trs()
                    .iter()
                    .cloned()
                    .map(|mut tr| {
                        tr.nextstate = state_map
                            [partition.borrow().get_class_id(tr.nextstate as usize)]
                        .unwrap() as StateId;
                        tr
                    })
                {
                    fst.add_tr(state_map[c].unwrap() as StateId, tr)?;
                }
            }
        }
    }

    fst.set_start(
        state_map[partition
            .borrow()
            .get_class_id(fst.start().unwrap() as usize)]
        .unwrap() as StateId,
    )?;

    connect(fst)?;

    Ok(())
}

// Compute the height (distance) to final state
pub fn fst_depth<W: Semiring, F: Fst<W>>(
    fst: &F,
    state_id_cour: StateId,
    accessible_states: &mut HashSet<StateId>,
    fully_examined_states: &mut HashSet<StateId>,
    heights: &mut Vec<i32>,
) -> Result<()> {
    accessible_states.insert(state_id_cour);

    for _ in heights.len()..=(state_id_cour as usize) {
        heights.push(-1);
    }

    let mut height_cur_state = 0;
    for tr in fst.get_trs(state_id_cour)?.trs() {
        let nextstate = tr.nextstate;

        if !accessible_states.contains(&nextstate) {
            fst_depth(
                fst,
                nextstate,
                accessible_states,
                fully_examined_states,
                heights,
            )?;
        }

        height_cur_state = max(height_cur_state, 1 + heights[nextstate as usize]);
    }
    fully_examined_states.insert(state_id_cour);

    heights[state_id_cour as usize] = height_cur_state;

    Ok(())
}

struct AcyclicMinimizer {
    partition: Rc<RefCell<Partition>>,
}

impl AcyclicMinimizer {
    pub fn new<W: Semiring, F: MutableFst<W>>(fst: &mut F) -> Result<Self> {
        let mut c = Self {
            partition: Rc::new(RefCell::new(Partition::empty_new())),
        };
        c.initialize(fst)?;
        c.refine(fst);
        Ok(c)
    }

    fn initialize<W: Semiring, F: MutableFst<W>>(&mut self, fst: &mut F) -> Result<()> {
        let mut accessible_state = HashSet::new();
        let mut fully_examined_states = HashSet::new();
        let mut heights = Vec::new();
        fst_depth(
            fst,
            fst.start().unwrap(),
            &mut accessible_state,
            &mut fully_examined_states,
            &mut heights,
        )?;
        self.partition.borrow_mut().initialize(heights.len());
        self.partition
            .borrow_mut()
            .allocate_classes((heights.iter().max().unwrap() + 1) as usize);
        for (s, h) in heights.iter().enumerate() {
            self.partition.borrow_mut().add(s, *h as usize);
        }
        Ok(())
    }

    fn refine<W: Semiring, F: MutableFst<W>>(&mut self, fst: &mut F) {
        let state_cmp = StateComparator {
            fst,
            partition: Rc::clone(&self.partition),
            w: PhantomData,
        };

        let height = self.partition.borrow().num_classes();
        for h in 0..height {
            // We need here a binary search tree in order to order the states id and create a partition.
            // For now uses the crate `stable_bst` which is quite old but seems to do the job
            // TODO: Bench the performances of the implementation. Maybe re-write it.
            let mut equiv_classes =
                TreeMap::<StateId, StateId, _>::with_comparator(|a: &StateId, b: &StateId| {
                    state_cmp.compare(*a, *b).unwrap()
                });

            let it_partition: Vec<_> = self.partition.borrow().iter(h).collect();
            equiv_classes.insert(it_partition[0] as StateId, h as StateId);

            for e in it_partition.iter().skip(1) {
                equiv_classes.get_or_insert(*e as StateId, || {
                    self.partition.borrow_mut().add_class() as StateId
                });
            }

            for s in it_partition {
                let old_class = self.partition.borrow().get_class_id(s);
                let new_class = *equiv_classes.get(&(s as StateId)).unwrap();

                if old_class != (new_class as usize) {
                    self.partition
                        .borrow_mut()
                        .move_element(s, new_class as usize);
                }
            }
        }
    }

    pub fn get_partition(self) -> Rc<RefCell<Partition>> {
        self.partition
    }
}

struct StateComparator<'a, W: Semiring, F: MutableFst<W>> {
    fst: &'a F,
    partition: Rc<RefCell<Partition>>,
    w: PhantomData<W>,
}

impl<'a, W: Semiring, F: MutableFst<W>> StateComparator<'a, W, F> {
    fn do_compare(&self, x: StateId, y: StateId) -> Result<bool> {
        let xfinal = self.fst.final_weight(x)?.unwrap_or_else(W::zero);
        let yfinal = self.fst.final_weight(y)?.unwrap_or_else(W::zero);

        if xfinal < yfinal {
            return Ok(true);
        } else if xfinal > yfinal {
            return Ok(false);
        }

        if self.fst.num_trs(x)? < self.fst.num_trs(y)? {
            return Ok(true);
        }
        if self.fst.num_trs(x)? > self.fst.num_trs(y)? {
            return Ok(false);
        }

        let it_x_owner = self.fst.get_trs(x)?;
        let it_x = it_x_owner.trs().iter();
        let it_y_owner = self.fst.get_trs(y)?;
        let it_y = it_y_owner.trs().iter();

        for (arc1, arc2) in it_x.zip(it_y) {
            if arc1.ilabel < arc2.ilabel {
                return Ok(true);
            }
            if arc1.ilabel > arc2.ilabel {
                return Ok(false);
            }
            let id_1 = self
                .partition
                .borrow()
                .get_class_id(arc1.nextstate as usize);
            let id_2 = self
                .partition
                .borrow()
                .get_class_id(arc2.nextstate as usize);
            if id_1 < id_2 {
                return Ok(true);
            }
            if id_1 > id_2 {
                return Ok(false);
            }
        }
        Ok(false)
    }

    pub fn compare(&self, x: StateId, y: StateId) -> Result<Ordering> {
        if x == y {
            return Ok(Ordering::Equal);
        }

        let x_y = self.do_compare(x, y).unwrap();
        let y_x = self.do_compare(y, x).unwrap();

        if !(x_y) && !(y_x) {
            return Ok(Ordering::Equal);
        }

        if x_y {
            Ok(Ordering::Less)
        } else {
            Ok(Ordering::Greater)
        }
    }
}

fn pre_partition<W: Semiring, F: MutableFst<W>>(
    fst: &F,
    partition: &Rc<RefCell<Partition>>,
    queue: &mut LifoQueue,
) {
    let mut next_class: StateId = 0;
    let num_states = fst.num_states();

    let mut state_to_initial_class: Vec<StateId> = vec![0; num_states];
    {
        let mut hash_to_class_nonfinal = HashMap::<Vec<Label>, StateId>::new();
        let mut hash_to_class_final = HashMap::<Vec<Label>, StateId>::new();

        for (s, state_to_initial_class_s) in state_to_initial_class
            .iter_mut()
            .enumerate()
            .take(num_states)
        {
            let this_map = if unsafe { fst.is_final_unchecked(s as StateId) } {
                &mut hash_to_class_final
            } else {
                &mut hash_to_class_nonfinal
            };

            let ilabels = fst
                .get_trs(s as StateId)
                .unwrap()
                .trs()
                .iter()
                .map(|e| e.ilabel)
                .dedup()
                .collect_vec();

            match this_map.entry(ilabels) {
                Entry::Occupied(e) => {
                    *state_to_initial_class_s = *e.get();
                }
                Entry::Vacant(e) => {
                    e.insert(next_class);
                    *state_to_initial_class_s = next_class;
                    next_class += 1;
                }
            };
        }
    }

    partition.borrow_mut().allocate_classes(next_class as usize);
    for (s, c) in state_to_initial_class.iter().enumerate().take(num_states) {
        partition.borrow_mut().add(s, *c as usize);
    }

    for c in 0..next_class {
        queue.enqueue(c);
    }
}

fn cyclic_minimize<W: Semiring, F: MutableFst<W>>(fst: &mut F) -> Result<Rc<RefCell<Partition>>> {
    // Initialize
    let mut tr: VectorFst<W::ReverseWeight> = reverse(fst)?;
    tr_sort(&mut tr, ILabelCompare {});

    let partition = Rc::new(RefCell::new(Partition::new(tr.num_states() - 1)));
    let mut queue = LifoQueue::default();
    pre_partition(fst, &partition, &mut queue);

    let comp = TrIterCompare {};

    let mut aiter_queue = BinaryHeap::new_by(|v1, v2| {
        if comp.compare(v1, v2) {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });

    // Compute
    while let Some(c) = queue.dequeue() {
        // Split
        for s in partition.borrow().iter(c as usize) {
            if tr.num_trs(s as StateId + 1)? > 0 {
                aiter_queue.push(TrsIterCollected {
                    idx: 0,
                    trs: tr.get_trs(s as StateId + 1)?,
                    w: PhantomData,
                });
            }
        }

        let mut prev_label = -1;
        while !aiter_queue.is_empty() {
            let mut aiter = aiter_queue.pop().unwrap();
            if aiter.done() {
                continue;
            }
            let tr = aiter.value().unwrap();
            let from_state = tr.nextstate - 1;
            let from_label = tr.ilabel;
            if prev_label != from_label as i32 {
                partition.borrow_mut().finalize_split(&mut Some(&mut queue));
            }
            let from_class = partition.borrow().get_class_id(from_state as usize);
            if partition.borrow().get_class_size(from_class) > 1 {
                partition.borrow_mut().split_on(from_state as usize);
            }
            prev_label = from_label as i32;
            aiter.next();
            if !aiter.done() {
                aiter_queue.push(aiter);
            }
        }

        partition.borrow_mut().finalize_split(&mut Some(&mut queue));
    }

    // Get Partition
    Ok(partition)
}

struct TrsIterCollected<W: Semiring, T: Trs<W>> {
    idx: usize,
    trs: T,
    w: PhantomData<W>,
}

impl<W: Semiring, T: Trs<W>> TrsIterCollected<W, T> {
    fn value(&self) -> Option<&Tr<W>> {
        self.trs.trs().get(self.idx)
    }

    fn done(&self) -> bool {
        self.idx >= self.trs.len()
    }

    fn next(&mut self) {
        self.idx += 1;
    }
}

#[derive(Debug, Clone)]
struct TrIterCompare {}

impl TrIterCompare {
    fn compare<W: Semiring, T: Trs<W>>(
        &self,
        x: &TrsIterCollected<W, T>,
        y: &TrsIterCollected<W, T>,
    ) -> bool {
        let xarc = x.value().unwrap();
        let yarc = y.value().unwrap();
        xarc.ilabel > yarc.ilabel
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use ::proptest::prelude::*;
    use algorithms::determinize::*;
    use std::sync::Arc;

    #[test]
    fn test_minimize_issue_158() {
        let text_fst = r#"0	5	101	101	0
0	4	100	100	0
0	3	99	99	0
0	2	98	98	0
0	1	97	97	0
1	10	101	101	0
1	9	100	100	0
1	8	99	99	0
1	7	98	98	0
1	6	97	97	0
2	11	101	101	0
2	10	100	100	0
2	9	99	99	0
2	8	98	98	0
2	7	97	97	0
3	11	100	100	0
3	10	99	99	0
3	9	98	98	0
3	8	97	97	0
4	11	99	99	0
4	10	98	98	0
4	9	97	97	0
5	11	98	98	0
5	10	97	97	0
6	15	101	101	0
6	14	100	100	0
6	13	99	99	0
6	12	98	98	0
7	16	101	101	0
7	15	100	100	0
7	14	99	99	0
7	13	98	98	0
7	12	97	97	0
8	16	100	100	0
8	15	99	99	0
8	14	98	98	0
8	13	97	97	0
9	16	99	99	0
9	15	98	98	0
9	14	97	97	0
10	16	98	98	0
10	15	97	97	0
11	16	97	97	0
12	17	101	101	0
13	17	100	100	0
14	17	99	99	0
15	17	98	98	0
16	17	97	97	0
17	18	32	32	0
18	0
        "#;
        let path = fst_path![97, 98, 97, 100, 32];
        let mut fst: VectorFst<TropicalWeight> = VectorFst::from_text_string(text_fst).unwrap();
        let accept1 = check_path_in_fst(&fst, &path);
        minimize(&mut fst).unwrap();
        let accept2 = check_path_in_fst(&fst, &path);

        assert_eq!(accept1, accept2);
    }

    proptest! {
        #[test]
        fn test_proptest_minimize_timeout(mut fst in any::<VectorFst::<TropicalWeight>>()) {
            let config = MinimizeConfig::default().with_allow_nondet(true);
            minimize_with_config(&mut fst, config).unwrap();
        }
    }

    proptest! {
        #[test]
        #[ignore] // falls into the same infinite loop as the timeout test
        fn test_minimize_proptest(mut fst in any::<VectorFst::<TropicalWeight>>()) {
            let det:VectorFst<_> = determinize_with_config(&fst, DeterminizeConfig::default().with_det_type(DeterminizeType::DeterminizeNonFunctional)).unwrap();
            let min_config = MinimizeConfig::default().with_allow_nondet(true);
            minimize_with_config(&mut fst, min_config).unwrap();
            let det_config = DeterminizeConfig::default().with_det_type(DeterminizeType::DeterminizeNonFunctional);
            let min_det:VectorFst<_> = determinize_with_config(&fst, det_config).unwrap();
            prop_assert!(isomorphic(&det, &min_det).unwrap())
        }
    }

    proptest! {
        #[test]
        fn test_proptest_minimize_keeps_symts(mut fst in any::<VectorFst::<TropicalWeight>>()) {
            let symt = Arc::new(SymbolTable::new());
            fst.set_input_symbols(Arc::clone(&symt));
            fst.set_output_symbols(Arc::clone(&symt));

            minimize_with_config(&mut fst, MinimizeConfig::default().with_allow_nondet(true)).unwrap();

            assert!(fst.input_symbols().is_some());
            assert!(fst.output_symbols().is_some());
        }
    }
}
