use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::marker::PhantomData;

use bimap::BiHashMap;
use failure::Fallible;

use crate::{EPS_LABEL, KDELTA, Label, StateId};
use crate::algorithms::{factor_weight, FactorWeightOptions, FactorWeightType, weight_convert};
use crate::algorithms::cache::{CacheImpl, FstImpl};
use crate::algorithms::factor_iterators::{GallicFactor, GallicFactorMin, GallicFactorRestrict};
use crate::algorithms::weight_converters::{FromGallicConverter, ToGallicConverter};
use crate::arc::Arc;
use crate::fst_impls::VectorFst;
use crate::fst_traits::{AllocableFst, CoreFst, ExpandedFst, Fst, MutableFst};
use crate::semirings::{
    DivideType, GallicWeight, GallicWeightLeft, GallicWeightMin, GallicWeightRestrict, Semiring,
    SemiringProperties, StringWeightLeft, StringWeightRestrict, WeaklyDivisibleSemiring,
    WeightQuantize,
};

/// Determinization type.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum DeterminizeType {
    /// Input transducer is known to be functional (or error).
    DeterminizeFunctional,
    /// Input transducer is NOT known to be functional.
    DeterminizeNonFunctional,
    /// Input transducer is not known to be functional but only keep the min of
    /// of ambiguous outputs.
    DeterminizeDisambiguate,
}

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

struct DeterminizeFsaImpl<'a, 'b, F: Fst, CD: CommonDivisor<F::W>>
where
    F::W: WeaklyDivisibleSemiring + WeightQuantize,
{
    fst: &'a F,
    cache_impl: CacheImpl<F::W>,
    state_table: BiHashMap<StateId, DeterminizeStateTuple<F::W>>,
    ghost: PhantomData<CD>,
    in_dist: Option<&'b [F::W]>,
    out_dist: Vec<F::W>,
}

impl<'a, 'b, F: Fst, CD: CommonDivisor<F::W>> FstImpl<F::W> for DeterminizeFsaImpl<'a, 'b, F, CD>
    where
        F::W: WeaklyDivisibleSemiring + WeightQuantize + 'static,
{
    fn cache_impl(&mut self) -> &mut CacheImpl<<F as CoreFst>::W> {
        &mut self.cache_impl
    }

    fn expand(&mut self, state: usize) -> Fallible<()> {
        // GetLabelMap
        let mut label_map: HashMap<Label, DeterminizeArc<F::W>> = HashMap::new();
        let src_tuple = self.state_table.get_by_left(&state).unwrap();
        for src_elt in src_tuple.subset.iter() {
            for arc in self.fst.arcs_iter(src_elt.state)? {
                let r = src_elt.weight.times(&arc.weight)?;

                let dest_elt = DeterminizeElement::new(arc.nextstate, r);

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

        for det_arc in label_map.values_mut() {
            self.norm_arc(det_arc)?;
        }

        for det_arc in label_map.values() {
            self.add_arc(state, det_arc)?;
        }

        Ok(())
    }

    fn compute_start(&mut self) -> Fallible<Option<usize>> {
        if let Some(start_state) = self.fst.start() {
            let elt = DeterminizeElement::new(start_state, F::W::one());
            let tuple = DeterminizeStateTuple {
                subset: WeightedSubset::from_vec(vec![elt]),
                filter_state: start_state,
            };
            return Ok(Some(self.find_state(&tuple)?));
        }
        Ok(None)
    }

    fn compute_final(&mut self, state: usize) -> Fallible<Option<<F as CoreFst>::W>> {
        let zero = F::W::zero();
        let tuple = self.state_table.get_by_left(&state).unwrap();
        let mut final_weight = F::W::zero();
        for det_elt in tuple.subset.iter() {
            final_weight.plus_assign(
                det_elt.weight.times(
                    self.fst
                        .final_weight(det_elt.state)?
                        .unwrap_or_else(|| &zero),
                )?,
            )?;
        }
        if final_weight.is_zero() {
            Ok(None)
        } else {
            Ok(Some(final_weight))
        }
    }
}


impl<'a, 'b, F: Fst, CD: CommonDivisor<F::W>> DeterminizeFsaImpl<'a, 'b, F, CD>
where
    F::W: WeaklyDivisibleSemiring + WeightQuantize,
{
    pub fn new(fst: &'a F, in_dist: Option<&'b [F::W]>) -> Fallible<Self> {
        if !fst.is_acceptor() {
            bail!("DeterminizeFsaImpl : expected acceptor as argument");
        }
        Ok(Self {
            fst,
            cache_impl: CacheImpl::new(),
            state_table: BiHashMap::new(),
            ghost: PhantomData,
            in_dist,
            out_dist: vec![],
        })
    }

    fn add_arc(&mut self, state: StateId, det_arc: &DeterminizeArc<F::W>) -> Fallible<()> {
        let arc = Arc::new(
            det_arc.label,
            det_arc.label,
            det_arc.weight.clone(),
            self.find_state(&det_arc.dest_tuple)?,
        );
        self.cache_impl.push_arc(state, arc)
    }

    fn norm_arc(&mut self, det_arc: &mut DeterminizeArc<F::W>) -> Fallible<()> {
        det_arc
            .dest_tuple
            .subset
            .pairs
            .sort_by(|a, b| a.state.cmp(&b.state));

        for dest_elt in det_arc.dest_tuple.subset.pairs.iter() {
            det_arc.weight = CD::common_divisor(&det_arc.weight, &dest_elt.weight)?;
        }

        let mut new_pairs = HashMap::new();
        for x in &mut det_arc.dest_tuple.subset.pairs {
            match new_pairs.entry(x.state) {
                Entry::Vacant(e) => {
                    e.insert(x.clone());
                }
                Entry::Occupied(mut e) => {
                    e.get_mut().weight.plus_assign(&x.weight)?;
                }
            };
        }

        det_arc.dest_tuple.subset.pairs = new_pairs.values().cloned().collect();

        for dest_elt in det_arc.dest_tuple.subset.pairs.iter_mut() {
            dest_elt.weight = dest_elt
                .weight
                .divide(&det_arc.weight, DivideType::DivideLeft)?;
            dest_elt.weight.quantize_assign(KDELTA)?;
        }

        Ok(())
    }

    fn find_state(&mut self, tuple: &DeterminizeStateTuple<F::W>) -> Fallible<StateId> {
        if !self.state_table.contains_right(tuple) {
            let n = self.state_table.len();
            self.state_table.insert(n, tuple.clone());
        }
        let s = *self.state_table.get_by_right(tuple).unwrap();
        if let Some(_in_dist) = self.in_dist.as_ref() {
            if self.out_dist.len() <= s {
                self.out_dist.push(self.compute_distance(&tuple.subset)?);
            }
        }
        Ok(s)
    }

    fn compute_distance(&self, subset: &WeightedSubset<F::W>) -> Fallible<F::W> {
        let mut outd = F::W::zero();
        let weight_zero = F::W::zero();
        for element in subset.iter() {
            let ind = if element.state < self.in_dist.as_ref().unwrap().len() {
                &self.in_dist.as_ref().unwrap()[element.state]
            } else {
                &weight_zero
            };
            outd.plus_assign(element.weight.times(ind)?)?;
        }
        Ok(outd)
    }

    pub fn compute_with_distance<F2: MutableFst<W = F::W> + ExpandedFst<W = F::W>>(
        &mut self,
    ) -> Fallible<(F2, Vec<F2::W>)>
    where
        F::W: 'static,
    {
        let dfst = self.compute()?;
        let out_dist = self.out_dist.clone();
        Ok((dfst, out_dist))
    }
}

pub fn determinize_with_distance<W, F1, F2>(ifst: &F1, in_dist: &[W]) -> Fallible<(F2, Vec<W>)>
where
    W: WeaklyDivisibleSemiring + WeightQuantize + 'static,
    F1: ExpandedFst<W = W>,
    F2: MutableFst<W = W> + ExpandedFst<W = W>,
{
    if !W::properties().contains(SemiringProperties::LEFT_SEMIRING) {
        bail!("determinize_fsa : weight must be left distributive")
    }
    let mut det_fsa_impl: DeterminizeFsaImpl<_, DefaultCommonDivisor> =
        DeterminizeFsaImpl::new(ifst, Some(in_dist))?;
    det_fsa_impl.compute_with_distance()
}

pub fn determinize_fsa<W, F1, F2, CD>(fst_in: &F1) -> Fallible<F2>
where
    W: WeaklyDivisibleSemiring + WeightQuantize + 'static,
    F1: Fst<W = W>,
    F2: MutableFst<W = W> + ExpandedFst<W = W>,
    CD: CommonDivisor<W>,
{
    if !W::properties().contains(SemiringProperties::LEFT_SEMIRING) {
        bail!("determinize_fsa : weight must be left distributive")
    }
    let mut det_fsa_impl: DeterminizeFsaImpl<_, CD> = DeterminizeFsaImpl::new(fst_in, None)?;
    det_fsa_impl.compute()
}

pub fn determinize_fst<W, F1, F2>(fst_in: &F1, det_type: DeterminizeType) -> Fallible<F2>
where
    W: WeaklyDivisibleSemiring + WeightQuantize + 'static,
    F1: ExpandedFst<W = W>,
    F2: MutableFst<W = W> + ExpandedFst<W = W> + AllocableFst,
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
            if !W::properties().contains(SemiringProperties::PATH) {
                bail!("determinize : weight needs to have the path property to disambiguate output")
            }
            let fsa: VectorFst<GallicWeightMin<W>> = weight_convert(fst_in, &mut to_gallic)?;
            let determinized_fsa: VectorFst<GallicWeightMin<W>> =
                determinize_fsa::<_, _, _, GallicCommonDivisor>(&fsa)?;
            let factored_determinized_fsa: VectorFst<GallicWeightMin<W>> =
                factor_weight::<_, _, GallicFactorMin<W>>(&determinized_fsa, factor_opts)?;
            weight_convert(&factored_determinized_fsa, &mut from_gallic)
        }
        DeterminizeType::DeterminizeFunctional => {
            let fsa: VectorFst<GallicWeightRestrict<W>> = weight_convert(fst_in, &mut to_gallic)?;
            let determinized_fsa: VectorFst<GallicWeightRestrict<W>> =
                determinize_fsa::<_, _, _, GallicCommonDivisor>(&fsa)?;
            let factored_determinized_fsa: VectorFst<GallicWeightRestrict<W>> =
                factor_weight::<_, _, GallicFactorRestrict<W>>(&determinized_fsa, factor_opts)?;
            weight_convert(&factored_determinized_fsa, &mut from_gallic)
        }
        DeterminizeType::DeterminizeNonFunctional => {
            let fsa: VectorFst<GallicWeight<W>> = weight_convert(fst_in, &mut to_gallic)?;
            let determinized_fsa: VectorFst<GallicWeight<W>> =
                determinize_fsa::<_, _, _, GallicCommonDivisor>(&fsa)?;
            let factored_determinized_fsa: VectorFst<GallicWeight<W>> =
                factor_weight::<_, _, GallicFactor<W>>(&determinized_fsa, factor_opts)?;
            weight_convert(&factored_determinized_fsa, &mut from_gallic)
        }
    }
}

/// This operations creates an equivalent FST that has the property that no
/// state has two transitions with the same input label. For this algorithm,
/// epsilon transitions are treated as regular symbols.
pub fn determinize<W, F1, F2>(fst_in: &F1, det_type: DeterminizeType) -> Fallible<F2>
where
    W: WeaklyDivisibleSemiring + WeightQuantize + 'static,
    F1: ExpandedFst<W = W>,
    F2: MutableFst<W = W> + ExpandedFst<W = W> + AllocableFst,
{
    if fst_in.is_acceptor() {
        determinize_fsa::<_, _, _, DefaultCommonDivisor>(fst_in)
    } else {
        determinize_fst(fst_in, det_type)
    }
}

#[cfg(test)]
mod tests {
    use crate::arc::Arc;
    use crate::fst_impls::VectorFst;
    use crate::semirings::TropicalWeight;

    use super::*;

    #[test]
    fn test_determinize() -> Fallible<()> {
        let mut input_fst = VectorFst::new();
        let s0 = input_fst.add_state();
        let s1 = input_fst.add_state();

        input_fst.set_start(s0)?;
        input_fst.set_final(s1, TropicalWeight::one())?;

        input_fst.add_arc(s0, Arc::new(1, 1, TropicalWeight::new(2.0), s1))?;
        input_fst.add_arc(s0, Arc::new(1, 1, TropicalWeight::new(2.0), s1))?;
        input_fst.add_arc(s0, Arc::new(1, 1, TropicalWeight::new(2.0), s1))?;

        let mut ref_fst = VectorFst::new();
        let s0 = ref_fst.add_state();
        let s1 = ref_fst.add_state();

        ref_fst.set_start(s0)?;
        ref_fst.set_final(s1, TropicalWeight::one())?;

        ref_fst.add_arc(s0, Arc::new(1, 1, TropicalWeight::new(2.0), s1))?;

        let determinized_fst: VectorFst<TropicalWeight> =
            determinize(&ref_fst, DeterminizeType::DeterminizeFunctional)?;

        assert_eq!(determinized_fst, ref_fst);
        Ok(())
    }

    #[test]
    fn test_determinize_2() -> Fallible<()> {
        let mut input_fst = VectorFst::new();
        let s0 = input_fst.add_state();
        let s1 = input_fst.add_state();
        let s2 = input_fst.add_state();
        let s3 = input_fst.add_state();

        input_fst.set_start(s0)?;
        input_fst.set_final(s3, TropicalWeight::one())?;

        input_fst.add_arc(s0, Arc::new(1, 1, TropicalWeight::new(2.0), s1))?;
        input_fst.add_arc(s0, Arc::new(1, 1, TropicalWeight::new(3.0), s2))?;

        input_fst.add_arc(s1, Arc::new(2, 2, TropicalWeight::new(4.0), s3))?;
        input_fst.add_arc(s2, Arc::new(2, 2, TropicalWeight::new(3.0), s3))?;

        let mut ref_fst = VectorFst::new();
        let s0 = ref_fst.add_state();
        let s1 = ref_fst.add_state();
        let s2 = ref_fst.add_state();

        ref_fst.set_start(s0)?;
        ref_fst.set_final(s2, TropicalWeight::one())?;

        ref_fst.add_arc(s0, Arc::new(1, 1, TropicalWeight::new(2.0), s1))?;
        ref_fst.add_arc(s1, Arc::new(2, 2, TropicalWeight::new(4.0), s2))?;

        let determinized_fst: VectorFst<TropicalWeight> =
            determinize(&ref_fst, DeterminizeType::DeterminizeFunctional)?;

        assert_eq!(determinized_fst, ref_fst);
        Ok(())
    }
}
