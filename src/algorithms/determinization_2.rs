use std::collections::hash_map::Entry;
use std::collections::HashMap;

use std::slice::Iter as IterSlice;

use bimap::BiHashMap;

use failure::Fallible;

use itertools::Itertools;

use crate::algorithms::cache::CacheImpl;
use crate::algorithms::factor_iterators::GallicFactor;
use crate::algorithms::factor_iterators::GallicFactorMin;
use crate::algorithms::factor_iterators::GallicFactorRestrict;
use crate::algorithms::factor_weight;
use crate::algorithms::weight_convert::weight_convert;
use crate::algorithms::weight_converters::FromGallicConverter;
use crate::algorithms::weight_converters::ToGallicConverter;
use crate::algorithms::DeterminizeType;
use crate::algorithms::{FactorWeightOptions, FactorWeightType};
use crate::fst_impls::VectorFst;
use crate::fst_traits::ExpandedFst;
use crate::fst_traits::Fst;
use crate::fst_traits::MutableFst;
use crate::semirings::GallicWeightMin;
use crate::semirings::{
    DivideType, GallicWeight, GallicWeightLeft, GallicWeightRestrict, Semiring, StringWeightLeft,
    StringWeightRestrict, WeaklyDivisibleSemiring, WeightQuantize,
};
use crate::Arc;
use crate::EPS_LABEL;
use crate::{Label, StateId, KDELTA};
use std::collections::HashSet;
use std::collections::VecDeque;
use std::marker::PhantomData;

pub trait CommonDivisor<W: Semiring> {
    fn common_divisor(w1: &W, w2: &W) -> Fallible<W>;
}

struct DefaultCommonDivisor {}

impl<W: Semiring> CommonDivisor<W> for DefaultCommonDivisor {
    fn common_divisor(w1: &W, w2: &W) -> Fallible<W> {
        w1.plus(w2)
    }
}

struct LabelCommonDivisor {}

macro_rules! impl_label_common_divisor {
    ($string_semiring: ident) => {
        impl CommonDivisor<$string_semiring> for LabelCommonDivisor {
            fn common_divisor(
                w1: &$string_semiring,
                w2: &$string_semiring,
            ) -> Fallible<$string_semiring> {
                let mut iter1 = w1.iter();
                let mut iter2 = w2.iter();
                if w1.value.is_empty_list() || w2.value.is_empty_list() {
                    Ok($string_semiring::one())
                } else if w1.value.is_infinity() {
                    Ok(iter2.next().unwrap().into())
                } else if w2.value.is_infinity() {
                    Ok(iter1.next().unwrap().into())
                } else {
                    let v1 = iter1.next().unwrap();
                    let v2 = iter2.next().unwrap();
                    if v1 == v2 {
                        Ok(v1.into())
                    } else {
                        Ok($string_semiring::one())
                    }
                }
            }
        }
    };
}

impl_label_common_divisor!(StringWeightLeft);
impl_label_common_divisor!(StringWeightRestrict);

struct GallicCommonDivisor {}

macro_rules! impl_gallic_common_divisor {
    ($gallic: ident) => {
        impl<W: Semiring> CommonDivisor<$gallic<W>> for GallicCommonDivisor {
            fn common_divisor(w1: &$gallic<W>, w2: &$gallic<W>) -> Fallible<$gallic<W>> {
                let v1 = LabelCommonDivisor::common_divisor(w1.value1(), w2.value1())?;
                let v2 = DefaultCommonDivisor::common_divisor(w1.value2(), w2.value2())?;
                Ok((v1, v2).into())
            }
        }
    };
}

impl_gallic_common_divisor!(GallicWeightLeft);
impl_gallic_common_divisor!(GallicWeightRestrict);
impl_gallic_common_divisor!(GallicWeightMin);

impl<W: Semiring> CommonDivisor<GallicWeight<W>> for GallicCommonDivisor {
    fn common_divisor(w1: &GallicWeight<W>, w2: &GallicWeight<W>) -> Fallible<GallicWeight<W>> {
        let mut weight = GallicWeightRestrict::zero();
        for w in w1.iter().chain(w2.iter()) {
            weight = GallicCommonDivisor::common_divisor(&weight, w)?;
        }
        if weight.is_zero() {
            Ok(GallicWeight::zero())
        } else {
            Ok(weight.into())
        }
    }
}

#[derive(PartialEq, Eq, Clone, Hash, PartialOrd, Debug)]
struct DeterminizeElement<W: Semiring> {
    state: StateId,
    weight: W,
}

impl<W: Semiring> DeterminizeElement<W> {
    pub fn new(state: StateId, weight: W) -> Self {
        DeterminizeElement { state, weight }
    }
}

#[derive(Default, PartialEq, Eq, Clone, Hash, PartialOrd, Debug)]
struct WeightedSubset<W: Semiring> {
    pairs: Vec<DeterminizeElement<W>>,
}

impl<W: Semiring> WeightedSubset<W> {
    pub fn from_vec(vec: Vec<DeterminizeElement<W>>) -> Self {
        WeightedSubset { pairs: vec }
    }

    //    pub fn add(&mut self, state: StateId, weight: W) {
    //        self.pairs.push(DeterminizeElement::new(state, weight));
    //    }

    pub fn iter(&self) -> impl Iterator<Item = &DeterminizeElement<W>> {
        self.pairs.iter()
    }
}

#[derive(Default, PartialEq, Eq, Clone, Hash, PartialOrd, Debug)]
struct DeterminizeStateTuple<W: Semiring> {
    subset: WeightedSubset<W>,
    filter_state: StateId,
}

#[derive(Default, PartialEq, Eq, Clone, Hash, PartialOrd, Debug)]
struct DeterminizeArc<W: Semiring> {
    label: Label,
    weight: W,
    dest_tuple: DeterminizeStateTuple<W>,
}

impl<W: Semiring> DeterminizeArc<W> {
    pub fn from_arc(arc: &Arc<W>, filter_state: StateId) -> Self {
        Self {
            label: arc.ilabel,
            weight: W::zero(),
            dest_tuple: DeterminizeStateTuple {
                subset: WeightedSubset::from_vec(vec![]),
                filter_state,
            },
        }
    }
}

struct DeterminizeFsaImpl<'a, F: Fst, CD: CommonDivisor<F::W>>
where
    F::W: WeaklyDivisibleSemiring + WeightQuantize,
{
    fst: &'a F,
    cache_impl: CacheImpl<F::W>,
    state_table: BiHashMap<StateId, DeterminizeStateTuple<F::W>>,
    ghost: PhantomData<CD>,
}

impl<'a, F: Fst, CD: CommonDivisor<F::W>> DeterminizeFsaImpl<'a, F, CD>
where
    F::W: WeaklyDivisibleSemiring + WeightQuantize,
{
    pub fn new(fst: &'a F) -> Self {
        Self {
            fst,
            cache_impl: CacheImpl::new(),
            state_table: BiHashMap::new(),
            ghost: PhantomData,
        }
    }
    pub fn arcs_iter(&mut self, state: StateId) -> Fallible<IterSlice<Arc<F::W>>> {
        if !self.cache_impl.expanded(state) {
            self.expand(state)?;
        }
        self.cache_impl.arcs_iter(state)
    }

    pub fn start(&mut self) -> Option<StateId> {
        if !self.cache_impl.has_start() {
            let start = self.compute_start();
            self.cache_impl.set_start(start);
        }
        self.cache_impl.start().unwrap()
    }

    pub fn final_weight(&mut self, state: StateId) -> Fallible<Option<&F::W>> {
        if !self.cache_impl.has_final(state) {
            let final_weight = self.compute_final(state)?;
            self.cache_impl.set_final_weight(state, final_weight)?;
        }
        self.cache_impl.final_weight(state)
    }

    fn compute_start(&mut self) -> Option<StateId> {
        if let Some(start_state) = self.fst.start() {
            let elt = DeterminizeElement::new(start_state, F::W::one());
            let mut tuple = DeterminizeStateTuple {
                subset: WeightedSubset::from_vec(vec![elt]),
                filter_state: start_state,
            };
            return Some(self.find_state(&tuple));
        }
        None
    }

    fn compute_final(&mut self, state: StateId) -> Fallible<Option<F::W>> {
        let tuple = self.state_table.get_by_left(&state).unwrap();
        println!("Final tuple {:?} {:?}", state, tuple);
        let mut final_weight = F::W::zero();
        for det_elt in tuple.subset.iter() {
            //            if let Some(final_weight_fst) = self.fst.final_weight(det_elt.state) {
            //                final_weight.plus_assign(det_elt.weight.times(final_weight_fst)?)?;
            //            }
            final_weight.plus_assign(
                det_elt.weight.times(
                    self.fst
                        .final_weight(det_elt.state)
                        .unwrap_or_else(F::W::zero),
                )?,
            )?;
        }
        if final_weight.is_zero() {
            Ok(None)
        } else {
            Ok(Some(final_weight))
        }
    }

    fn expand(&mut self, state: StateId) -> Fallible<()> {
        println!("[Expand Start] s = {}", state);
        // GetLabelMap
        let mut label_map: HashMap<Label, DeterminizeArc<F::W>> = HashMap::new();
        let src_tuple = self.state_table.get_by_left(&state).unwrap();
        for src_elt in src_tuple.subset.iter() {
            for arc in self.fst.arcs_iter(src_elt.state)? {
                println!("{:?}", arc);
                let dest_elt =
                    DeterminizeElement::new(arc.nextstate, src_elt.weight.times(&arc.weight)?);

                // Filter Arc
                match label_map.entry(arc.ilabel) {
                    Entry::Occupied(_) => {}
                    Entry::Vacant(e) => {
                        e.insert(DeterminizeArc::from_arc(arc, 0));
                    }
                };
                label_map
                    .get_mut(&arc.ilabel)
                    .unwrap()
                    .dest_tuple
                    .subset
                    .pairs
                    .push(dest_elt);
            }
        }
        println!("Len Label map = {:#?}", &label_map);
        for det_arc in label_map.values_mut() {
            self.norm_arc(det_arc)?;
        }

        println!("Len Label map = {:#?}", label_map.len());
        for det_arc in label_map.values() {
            self.add_arc(state, det_arc)?;
        }
        self.cache_impl.mark_expanded(state);
        Ok(())
    }

    fn add_arc(&mut self, state: StateId, det_arc: &DeterminizeArc<F::W>) -> Fallible<()> {
        let arc = Arc::new(
            det_arc.label,
            det_arc.label,
            det_arc.weight.clone(),
            self.find_state(&det_arc.dest_tuple),
        );
        self.cache_impl.push_arc(state, arc)
    }

    fn norm_arc(&mut self, det_arc: &mut DeterminizeArc<F::W>) -> Fallible<()> {
        det_arc
            .dest_tuple
            .subset
            .pairs
            .sort_by(|a, b| a.state.partial_cmp(&b.state).unwrap());

        for dest_elt in det_arc.dest_tuple.subset.pairs.iter() {
            det_arc.weight = CD::common_divisor(&det_arc.weight, &dest_elt.weight)?;
        }

        det_arc.dest_tuple.subset.pairs = det_arc
            .dest_tuple
            .subset
            .pairs
            .iter()
            .cloned()
            .coalesce(|x, mut y| {
                if x.state == y.state {
                    y.weight.plus_assign(&x.weight);
                    Ok(y)
                } else {
                    Err((x, y))
                }
            })
            .collect();

        for dest_elt in det_arc.dest_tuple.subset.pairs.iter_mut() {
            dest_elt.weight = dest_elt
                .weight
                .divide(&det_arc.weight, DivideType::DivideLeft)?;
            dest_elt.weight.quantize_assign(KDELTA)?;
        }

        Ok(())
    }

    fn find_state(&mut self, tuple: &DeterminizeStateTuple<F::W>) -> StateId {
        if !self.state_table.contains_right(tuple) {
            let n = self.state_table.len();
            self.state_table.insert(n, tuple.clone());
        }
        *self.state_table.get_by_right(tuple).unwrap()
    }

    pub fn compute<F2: MutableFst<W = F::W> + ExpandedFst<W = F::W>>(&mut self) -> Fallible<F2>
    where
        F::W: 'static,
    {
        let start_state = self.start();
        let mut fst_out = F2::new();
        if start_state.is_none() {
            return Ok(fst_out);
        }
        let start_state = start_state.unwrap();
        for _ in 0..=start_state {
            fst_out.add_state();
        }
        fst_out.set_start(start_state)?;
        let mut queue = VecDeque::new();
        let mut visited_states = HashSet::new();
        visited_states.insert(start_state);
        queue.push_back(start_state);
        while !queue.is_empty() {
            let s = queue.pop_front().unwrap();
            for arc in self.arcs_iter(s)? {
                if !visited_states.contains(&arc.nextstate) {
                    queue.push_back(arc.nextstate);
                    visited_states.insert(arc.nextstate);
                }
                let n = fst_out.num_states();
                for _ in n..=arc.nextstate {
                    fst_out.add_state();
                }
                fst_out.add_arc(s, arc.clone())?;
            }
            if let Some(f_w) = self.final_weight(s)? {
                fst_out.set_final(s, f_w.clone())?;
            }
        }
        Ok(fst_out)
    }
}

pub fn determinize_fsa<W, F1, F2, CD>(fst_in: &F1) -> Fallible<F2>
where
    W: WeaklyDivisibleSemiring + WeightQuantize + 'static,
    F1: Fst<W = W>,
    F2: MutableFst<W = W> + ExpandedFst<W = W>,
    CD: CommonDivisor<W>,
{
    let mut det_fsa_impl: DeterminizeFsaImpl<_, CD> = DeterminizeFsaImpl::new(fst_in);
    det_fsa_impl.compute()
}

pub fn determinize_fst<W, F1, F2>(fst_in: &F1, det_type: DeterminizeType) -> Fallible<F2>
where
    W: WeaklyDivisibleSemiring + WeightQuantize + 'static,
    F1: ExpandedFst<W = W>,
    F2: MutableFst<W = W> + ExpandedFst<W = W>,
{
    let mut to_gallic = ToGallicConverter {};
    let mut from_gallic = FromGallicConverter {
        superfinal_label: EPS_LABEL,
    };

    let factor_opts = FactorWeightOptions {
        delta: KDELTA,
        mode: FactorWeightType::FACTOR_FINAL_WEIGHTS,
        final_ilabel: 0,
        final_olabel: 0,
        increment_final_ilabel: false,
        increment_final_olabel: false,
    };

    match det_type {
        DeterminizeType::DeterminizeDisambiguate => {
            let fsa: VectorFst<GallicWeightMin<W>> = weight_convert(fst_in, &mut to_gallic)?;
            let determinized_fsa: VectorFst<GallicWeightMin<W>> =
                determinize_fsa::<_, _, _, GallicCommonDivisor>(&fsa)?;
            println!("determinized_fsa=\n{}", &determinized_fsa);
            let factored_determinized_fsa: VectorFst<GallicWeightMin<W>> =
                factor_weight::<_, _, GallicFactorMin<W>>(&determinized_fsa, factor_opts)?;
            let determinized_fst = weight_convert(&factored_determinized_fsa, &mut from_gallic);
            determinized_fst
        }
        DeterminizeType::DeterminizeFunctional => {
            let fsa: VectorFst<GallicWeightRestrict<W>> = weight_convert(fst_in, &mut to_gallic)?;
            let determinized_fsa: VectorFst<GallicWeightRestrict<W>> =
                determinize_fsa::<_, _, _, GallicCommonDivisor>(&fsa)?;
            println!("determinized_fsa=\n{}", &determinized_fsa);
            let factored_determinized_fsa: VectorFst<GallicWeightRestrict<W>> =
                factor_weight::<_, _, GallicFactorRestrict<W>>(&determinized_fsa, factor_opts)?;
            let determinized_fst = weight_convert(&factored_determinized_fsa, &mut from_gallic);
            determinized_fst
        }
        DeterminizeType::DeterminizeNonFunctional => {
            let fsa: VectorFst<GallicWeight<W>> = weight_convert(fst_in, &mut to_gallic)?;
            let determinized_fsa: VectorFst<GallicWeight<W>> =
                determinize_fsa::<_, _, _, GallicCommonDivisor>(&fsa)?;
            println!("determinized_fsa=\n{}", &determinized_fsa);
            let factored_determinized_fsa: VectorFst<GallicWeight<W>> =
                factor_weight::<_, _, GallicFactor<W>>(&determinized_fsa, factor_opts)?;
            let determinized_fst = weight_convert(&factored_determinized_fsa, &mut from_gallic);
            determinized_fst
        }
    }
}

pub fn determinize<W, F1, F2>(fst_in: &F1, det_type: DeterminizeType) -> Fallible<F2>
where
    W: WeaklyDivisibleSemiring + WeightQuantize + 'static,
    F1: ExpandedFst<W = W>,
    F2: MutableFst<W = W> + ExpandedFst<W = W>,
{
    if fst_in.is_acceptor() {
        determinize_fsa::<_, _, _, DefaultCommonDivisor>(fst_in)
    } else {
        determinize_fst(fst_in, det_type)
    }
}
