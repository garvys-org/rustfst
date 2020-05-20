use std::cmp::max;
use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;
use std::marker::PhantomData;

use anyhow::Result;
use binary_heap_plus::BinaryHeap;
use stable_bst::TreeMap;

use crate::algorithms::factor_weight::factor_iterators::GallicFactorLeft;
use crate::algorithms::factor_weight::{factor_weight, FactorWeightOptions, FactorWeightType};
use crate::algorithms::partition::Partition;
use crate::algorithms::queues::LifoQueue;
use crate::algorithms::reverse;
use crate::algorithms::tr_compares::ilabel_compare;
use crate::algorithms::tr_mappers::QuantizeMapper;
use crate::algorithms::tr_unique;
use crate::algorithms::weight_converters::{FromGallicConverter, ToGallicConverter};
use crate::algorithms::Queue;
use crate::algorithms::{
    connect,
    encode::{decode, encode},
    push_weights, tr_map, tr_sort, weight_convert, ReweightType,
};
use crate::fst_impls::VectorFst;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{AllocableFst, CoreFst, ExpandedFst, Fst, MutableFst};
use crate::semirings::{
    GallicWeightLeft, Semiring, SemiringProperties, WeaklyDivisibleSemiring, WeightQuantize,
};
use crate::Tr;
use crate::EPS_LABEL;
use crate::KDELTA;
use crate::NO_STATE_ID;
use crate::{StateId, Trs};

/// In place minimization of deterministic weighted automata and transducers,
/// and also non-deterministic ones if they use an idempotent semiring.
/// For transducers, the algorithm produces a compact factorization of the minimal transducer.
pub fn minimize<W, F>(ifst: &mut F, allow_nondet: bool) -> Result<()>
where
    F: MutableFst<W> + ExpandedFst<W> + AllocableFst<W>,
    W: WeaklyDivisibleSemiring + WeightQuantize + 'static,
    <W as Semiring>::ReverseWeight: 'static,
{
    let props = ifst.properties()?;

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
        push_weights(&mut gfst, ReweightType::ReweightToInitial, false)?;
        let mut quantize_mapper = QuantizeMapper {};
        tr_map(&mut gfst, &mut quantize_mapper)?;
        let encode_table = encode(&mut gfst, true, true)?;
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
        push_weights(ifst, ReweightType::ReweightToInitial, false)?;
        let mut quantize_mapper = QuantizeMapper {};
        tr_map(ifst, &mut quantize_mapper)?;
        let encode_table = encode(ifst, true, true)?;
        acceptor_minimize(ifst, allow_acyclic_minimization)?;
        decode(ifst, encode_table)
    } else {
        // Unweighted acceptor
        acceptor_minimize(ifst, allow_acyclic_minimization)
    }
}

fn acceptor_minimize<W: Semiring, F: MutableFst<W> + ExpandedFst<W>>(
    ifst: &mut F,
    allow_acyclic_minimization: bool,
) -> Result<()> {
    let props = ifst.properties()?;
    if !props.contains(FstProperties::ACCEPTOR | FstProperties::UNWEIGHTED) {
        bail!("FST is not an unweighted acceptor");
    }

    connect(ifst)?;

    if ifst.num_states() == 0 {
        return Ok(());
    }

    if allow_acyclic_minimization && props.contains(FstProperties::ACYCLIC) {
        // Acyclic minimization
        tr_sort(ifst, ilabel_compare);
        let minimizer = AcyclicMinimizer::new(ifst)?;
        merge_states(minimizer.get_partition(), ifst)?;
    } else {
        let p = cyclic_minimize(ifst)?;
        merge_states(p, ifst)?;
    }

    tr_unique(ifst);

    Ok(())
}

fn merge_states<W: Semiring, F: MutableFst<W>>(partition: Partition, fst: &mut F) -> Result<()> {
    let mut state_map = vec![None; partition.num_classes()];
    for (i, s) in state_map
        .iter_mut()
        .enumerate()
        .take(partition.num_classes())
    {
        *s = partition.iter(i).next();
    }

    for c in 0..partition.num_classes() {
        for s in partition.iter(c) {
            if s == state_map[c].unwrap() {
                for tr in fst.tr_iter_mut(s)? {
                    tr.nextstate = state_map[partition.get_class_id(tr.nextstate)].unwrap();
                }
            } else {
                let trs: Vec<_> = fst
                    .get_trs(s)?
                    .trs()
                    .iter()
                    .cloned()
                    .map(|mut tr| {
                        tr.nextstate = state_map[partition.get_class_id(tr.nextstate)].unwrap();
                        tr
                    })
                    .collect();
                for tr in trs.into_iter() {
                    fst.add_tr(state_map[c].unwrap(), tr)?;
                }
            }
        }
    }

    fst.set_start(state_map[partition.get_class_id(fst.start().unwrap())].unwrap())?;
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

    for _ in heights.len()..=state_id_cour {
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

        height_cur_state = max(height_cur_state, 1 + heights[nextstate]);
    }
    fully_examined_states.insert(state_id_cour);

    heights[state_id_cour] = height_cur_state;

    Ok(())
}

struct AcyclicMinimizer {
    partition: Partition,
}

impl AcyclicMinimizer {
    pub fn new<W: Semiring, F: MutableFst<W>>(fst: &mut F) -> Result<Self> {
        let mut c = Self {
            partition: Partition::empty_new(),
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
        self.partition.initialize(heights.len());
        self.partition
            .allocate_classes((heights.iter().max().unwrap() + 1) as usize);
        for (s, h) in heights.iter().enumerate() {
            self.partition.add(s, *h as usize);
        }
        Ok(())
    }

    fn refine<W: Semiring, F: MutableFst<W>>(&mut self, fst: &mut F) {
        let state_cmp = StateComparator {
            fst,
            // This clone is necessary for the moment because the partition is modified while
            // still needing the StateComparator.
            // TODO: Find a way to remove the clone.
            partition: self.partition.clone(),
            w: PhantomData,
        };

        let height = self.partition.num_classes();
        for h in 0..height {
            // We need here a binary search tree in order to order the states id and create a partition.
            // For now uses the crate `stable_bst` which is quite old but seems to do the job
            // TODO: Bench the performances of the implementation. Maybe re-write it.
            let mut equiv_classes =
                TreeMap::<StateId, StateId, _>::with_comparator(|a: &usize, b: &usize| {
                    state_cmp.compare(*a, *b).unwrap()
                });

            let it_partition: Vec<_> = self.partition.iter(h).collect();
            equiv_classes.insert(it_partition[0], h);

            let mut classes_to_add = vec![];
            for e in it_partition.iter().skip(1) {
                // TODO: Remove double lookup
                if equiv_classes.contains_key(e) {
                    equiv_classes.insert(*e, NO_STATE_ID);
                } else {
                    classes_to_add.push(e);
                    equiv_classes.insert(*e, NO_STATE_ID);
                }
            }

            for v in classes_to_add {
                equiv_classes.insert(*v, self.partition.add_class());
            }

            for s in it_partition {
                let old_class = self.partition.get_class_id(s);
                let new_class = *equiv_classes.get(&s).unwrap();
                if new_class == NO_STATE_ID {
                    // The behaviour here is a bit different compared to the c++ because here
                    // when inserting an equivalent key it modifies the key
                    // which is not the case in c++.
                    continue;
                }
                if old_class != (new_class as usize) {
                    self.partition.move_element(s, new_class as usize);
                }
            }
        }
    }

    pub fn get_partition(self) -> Partition {
        self.partition
    }
}

struct StateComparator<'a, W: Semiring, F: MutableFst<W>> {
    fst: &'a F,
    partition: Partition,
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
            let id_1 = self.partition.get_class_id(arc1.nextstate);
            let id_2 = self.partition.get_class_id(arc2.nextstate);
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
    partition: &mut Partition,
    queue: &mut LifoQueue,
) {
    let mut next_class: StateId = 0;
    let num_states = fst.num_states();
    let mut state_to_initial_class: Vec<StateId> = vec![0; num_states];
    {
        let mut hash_to_class_nonfinal = HashMap::<Vec<usize>, StateId>::new();
        let mut hash_to_class_final = HashMap::<Vec<usize>, StateId>::new();

        for s in 0..num_states {
            let ilabels: Vec<usize> = unsafe { fst.get_trs_unchecked(s).trs().iter() }
                .map(|tr| tr.ilabel)
                .collect();

            let this_map = if unsafe { fst.is_final_unchecked(s) } {
                &mut hash_to_class_final
            } else {
                &mut hash_to_class_nonfinal
            };

            match this_map.entry(ilabels) {
                Entry::Occupied(e) => {
                    state_to_initial_class[s] = *e.get();
                }
                Entry::Vacant(e) => {
                    e.insert(next_class);
                    state_to_initial_class[s] = next_class;
                    next_class += 1;
                }
            };
        }
    }
    partition.allocate_classes(next_class);
    for (s, c) in state_to_initial_class.iter().enumerate().take(num_states) {
        partition.add(s, *c);
    }

    for c in 0..next_class {
        queue.enqueue(c);
    }
}

fn cyclic_minimize<W: Semiring, F: MutableFst<W>>(fst: &mut F) -> Result<Partition> {
    // Initialize
    let mut tr: VectorFst<W::ReverseWeight> = reverse(fst)?;
    tr_sort(&mut tr, ilabel_compare);
    let mut partition = Partition::new(tr.num_states() - 1);
    let mut queue = LifoQueue::default();
    pre_partition(fst, &mut partition, &mut queue);

    // Compute
    while !queue.is_empty() {
        let c = queue.head().unwrap();
        queue.dequeue();

        // TODO: Avoid this clone :o
        // Here we need to pointer to the partition that is valid even if the partition changes.
        let comp = TrIterCompare {
            partition: partition.clone(),
        };
        let mut aiter_queue = BinaryHeap::new_by(|v1, v2| {
            if comp.compare(v1, v2) {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });

        // Split
        for s in partition.iter(c) {
            if tr.num_trs(s + 1)? > 0 {
                aiter_queue.push(TrsIterCollected {
                    idx: 0,
                    trs: tr.get_trs(s + 1)?,
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
            let tr = aiter.peek().unwrap();
            let from_state = tr.nextstate - 1;
            let from_label = tr.ilabel;
            if prev_label != from_label as i32 {
                partition.finalize_split(&mut Some(&mut queue));
            }
            let from_class = partition.get_class_id(from_state);
            if partition.get_class_size(from_class) > 1 {
                partition.split_on(from_state);
            }
            prev_label = from_label as i32;
            aiter.next();
            if !aiter.done() {
                aiter_queue.push(aiter);
            }
        }

        partition.finalize_split(&mut Some(&mut queue));
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
    fn peek(&self) -> Option<&Tr<W>> {
        self.trs.trs().get(self.idx)
    }

    fn done(&self) -> bool {
        self.idx >= self.trs.len()
    }

    fn next(&mut self) {
        self.idx += 1;
    }
}

#[derive(Clone)]
struct TrIterCompare {
    partition: Partition,
}

impl TrIterCompare {
    fn compare<W: Semiring, T: Trs<W>>(
        &self,
        x: &TrsIterCollected<W, T>,
        y: &TrsIterCollected<W, T>,
    ) -> bool
    where
        W: Semiring,
    {
        let xarc = x.peek().unwrap();
        let yarc = y.peek().unwrap();
        xarc.ilabel > yarc.ilabel
    }
}
