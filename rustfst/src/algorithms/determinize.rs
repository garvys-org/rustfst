use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

use anyhow::Result;

use crate::algorithms::cache::{CacheImpl, FstImpl, StateTable};
use crate::algorithms::factor_iterators::{GallicFactor, GallicFactorMin, GallicFactorRestrict};
use crate::algorithms::weight_converters::{FromGallicConverter, ToGallicConverter};
use crate::algorithms::{factor_weight, weight_convert, FactorWeightOptions, FactorWeightType};
use crate::fst_impls::VectorFst;
use crate::fst_traits::{AllocableFst, CoreFst, ExpandedFst, Fst, MutableFst};
use crate::semirings::{
    DivideType, GallicWeight, GallicWeightLeft, GallicWeightMin, GallicWeightRestrict, Semiring,
    SemiringProperties, StringWeightLeft, StringWeightRestrict, WeaklyDivisibleSemiring,
    WeightQuantize,
};
use crate::tr::Tr;
use crate::{Label, StateId, Trs, EPS_LABEL, KDELTA};

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

pub trait CommonDivisor<W: Semiring>: PartialEq + Debug {
    fn common_divisor(w1: &W, w2: &W) -> Result<W>;
}

#[derive(PartialEq, Debug)]
struct DefaultCommonDivisor {}

impl<W: Semiring> CommonDivisor<W> for DefaultCommonDivisor {
    fn common_divisor(w1: &W, w2: &W) -> Result<W> {
        w1.plus(w2)
    }
}

#[derive(PartialEq, Debug)]
struct LabelCommonDivisor {}

macro_rules! impl_label_common_divisor {
    ($string_semiring: ident) => {
        impl CommonDivisor<$string_semiring> for LabelCommonDivisor {
            fn common_divisor(
                w1: &$string_semiring,
                w2: &$string_semiring,
            ) -> Result<$string_semiring> {
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

#[derive(Debug, PartialEq)]
struct GallicCommonDivisor {}

macro_rules! impl_gallic_common_divisor {
    ($gallic: ident) => {
        impl<W: Semiring> CommonDivisor<$gallic<W>> for GallicCommonDivisor {
            fn common_divisor(w1: &$gallic<W>, w2: &$gallic<W>) -> Result<$gallic<W>> {
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
    fn common_divisor(w1: &GallicWeight<W>, w2: &GallicWeight<W>) -> Result<GallicWeight<W>> {
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
struct DeterminizeTr<W: Semiring> {
    label: Label,
    weight: W,
    dest_tuple: DeterminizeStateTuple<W>,
}

impl<W: Semiring> DeterminizeTr<W> {
    pub fn from_tr(tr: &Tr<W>, filter_state: StateId) -> Self {
        Self {
            label: tr.ilabel,
            weight: W::zero(),
            dest_tuple: DeterminizeStateTuple {
                subset: WeightedSubset::from_vec(vec![]),
                filter_state,
            },
        }
    }
}

#[derive(PartialEq, Debug)]
struct DeterminizeFsaImpl<'a, 'b, W: Semiring, F: Fst<W>, CD: CommonDivisor<W>> {
    fst: &'a F,
    cache_impl: CacheImpl<W>,
    state_table: StateTable<DeterminizeStateTuple<W>>,
    ghost: PhantomData<CD>,
    in_dist: Option<&'b [W]>,
    out_dist: Vec<W>,
}

impl<'a, 'b, W, F: Fst<W>, CD: CommonDivisor<W>> FstImpl for DeterminizeFsaImpl<'a, 'b, W, F, CD>
where
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
{
    type W = W;
    fn cache_impl_mut(&mut self) -> &mut CacheImpl<W> {
        &mut self.cache_impl
    }
    fn cache_impl_ref(&self) -> &CacheImpl<W> {
        &self.cache_impl
    }

    fn expand(&mut self, state: usize) -> Result<()> {
        // GetLabelMap
        let mut label_map: HashMap<Label, DeterminizeTr<W>> = HashMap::new();
        let src_tuple = self.state_table.find_tuple(state);
        for src_elt in src_tuple.subset.iter() {
            for tr in self.fst.get_trs(src_elt.state)?.trs() {
                let r = src_elt.weight.times(&tr.weight)?;

                let dest_elt = DeterminizeElement::new(tr.nextstate, r);

                // Filter Tr
                match label_map.entry(tr.ilabel) {
                    Entry::Occupied(_) => {}
                    Entry::Vacant(e) => {
                        e.insert(DeterminizeTr::from_tr(tr, 0));
                    }
                };

                label_map
                    .get_mut(&tr.ilabel)
                    .unwrap()
                    .dest_tuple
                    .subset
                    .pairs
                    .push(dest_elt);
            }
        }
        drop(src_tuple);

        for det_tr in label_map.values_mut() {
            self.norm_tr(det_tr)?;
        }

        for det_tr in label_map.values() {
            self.add_tr(state, det_tr)?;
        }

        Ok(())
    }

    fn compute_start(&mut self) -> Result<Option<usize>> {
        if let Some(start_state) = self.fst.start() {
            let elt = DeterminizeElement::new(start_state, W::one());
            let tuple = DeterminizeStateTuple {
                subset: WeightedSubset::from_vec(vec![elt]),
                filter_state: start_state,
            };
            return Ok(Some(self.find_state(&tuple)?));
        }
        Ok(None)
    }

    fn compute_final(&mut self, state: usize) -> Result<Option<W>> {
        let zero = W::zero();
        let tuple = self.state_table.find_tuple(state);
        let mut final_weight = W::zero();
        for det_elt in tuple.subset.iter() {
            final_weight.plus_assign(
                det_elt.weight.times(
                    self.fst
                        .final_weight(det_elt.state)?
                        .unwrap_or_else(W::zero),
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

impl<'a, 'b, W, F: Fst<W>, CD: CommonDivisor<W>> DeterminizeFsaImpl<'a, 'b, W, F, CD>
where
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
{
    pub fn new(fst: &'a F, in_dist: Option<&'b [W]>) -> Result<Self> {
        if !fst.is_acceptor() {
            bail!("DeterminizeFsaImpl : expected acceptor as argument");
        }
        Ok(Self {
            fst,
            cache_impl: CacheImpl::new(),
            state_table: StateTable::new(),
            ghost: PhantomData,
            in_dist,
            out_dist: vec![],
        })
    }

    fn add_tr(&mut self, state: StateId, det_tr: &DeterminizeTr<W>) -> Result<()> {
        let tr = Tr::new(
            det_tr.label,
            det_tr.label,
            det_tr.weight.clone(),
            self.find_state(&det_tr.dest_tuple)?,
        );
        self.cache_impl.push_tr(state, tr)
    }

    fn norm_tr(&mut self, det_tr: &mut DeterminizeTr<W>) -> Result<()> {
        det_tr
            .dest_tuple
            .subset
            .pairs
            .sort_by(|a, b| a.state.cmp(&b.state));

        for dest_elt in det_tr.dest_tuple.subset.pairs.iter() {
            det_tr.weight = CD::common_divisor(&det_tr.weight, &dest_elt.weight)?;
        }

        let mut new_pairs = HashMap::new();
        for x in &mut det_tr.dest_tuple.subset.pairs {
            match new_pairs.entry(x.state) {
                Entry::Vacant(e) => {
                    e.insert(x.clone());
                }
                Entry::Occupied(mut e) => {
                    e.get_mut().weight.plus_assign(&x.weight)?;
                }
            };
        }

        det_tr.dest_tuple.subset.pairs = new_pairs.values().cloned().collect();

        for dest_elt in det_tr.dest_tuple.subset.pairs.iter_mut() {
            dest_elt.weight = dest_elt
                .weight
                .divide(&det_tr.weight, DivideType::DivideLeft)?;
            dest_elt.weight.quantize_assign(KDELTA)?;
        }

        Ok(())
    }

    fn find_state(&mut self, tuple: &DeterminizeStateTuple<W>) -> Result<StateId> {
        let s = self.state_table.find_id_from_ref(&tuple);
        if let Some(_in_dist) = self.in_dist.as_ref() {
            if self.out_dist.len() <= s {
                self.out_dist.push(self.compute_distance(&tuple.subset)?);
            }
        }
        Ok(s)
    }

    fn compute_distance(&self, subset: &WeightedSubset<W>) -> Result<W> {
        let mut outd = W::zero();
        let weight_zero = W::zero();
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

    pub fn compute_with_distance<F2: MutableFst<W>>(&mut self) -> Result<(F2, Vec<W>)> {
        let dfst = self.compute()?;
        let out_dist = self.out_dist.clone();
        Ok((dfst, out_dist))
    }
}

pub fn determinize_with_distance<W, F1, F2>(ifst: &F1, in_dist: &[W]) -> Result<(F2, Vec<W>)>
where
    W: WeaklyDivisibleSemiring + WeightQuantize + 'static,
    F1: ExpandedFst<W>,
    F2: MutableFst<W> + ExpandedFst<W>,
{
    if !W::properties().contains(SemiringProperties::LEFT_SEMIRING) {
        bail!("determinize_fsa : weight must be left distributive")
    }
    let mut det_fsa_impl: DeterminizeFsaImpl<_, _, DefaultCommonDivisor> =
        DeterminizeFsaImpl::new(ifst, Some(in_dist))?;
    det_fsa_impl.compute_with_distance()
}

pub fn determinize_fsa<W, F1, F2, CD>(fst_in: &F1) -> Result<F2>
where
    W: WeaklyDivisibleSemiring + WeightQuantize + 'static,
    F1: Fst<W>,
    F2: MutableFst<W>,
    CD: CommonDivisor<W>,
{
    if !W::properties().contains(SemiringProperties::LEFT_SEMIRING) {
        bail!("determinize_fsa : weight must be left distributive")
    }
    let mut det_fsa_impl: DeterminizeFsaImpl<_, _, CD> = DeterminizeFsaImpl::new(fst_in, None)?;
    det_fsa_impl.compute()
}

pub fn determinize_fst<W, F1, F2>(fst_in: &F1, det_type: DeterminizeType) -> Result<F2>
where
    W: WeaklyDivisibleSemiring + WeightQuantize + 'static,
    F1: ExpandedFst<W>,
    F2: MutableFst<W> + AllocableFst<W>,
{
    let mut to_gallic = ToGallicConverter {};
    let mut from_gallic = FromGallicConverter {
        superfinal_label: EPS_LABEL,
    };

    let factor_opts = FactorWeightOptions {
        delta: KDELTA,
        mode: FactorWeightType::FACTOR_FINAL_WEIGHTS,
        final_ilabel: EPS_LABEL,
        final_olabel: EPS_LABEL,
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
                factor_weight::<_, VectorFst<GallicWeightMin<W>>, _, _, GallicFactorMin<W>>(
                    &determinized_fsa,
                    factor_opts,
                )?;
            weight_convert(&factored_determinized_fsa, &mut from_gallic)
        }
        DeterminizeType::DeterminizeFunctional => {
            let fsa: VectorFst<GallicWeightRestrict<W>> = weight_convert(fst_in, &mut to_gallic)?;
            let determinized_fsa: VectorFst<GallicWeightRestrict<W>> =
                determinize_fsa::<_, _, _, GallicCommonDivisor>(&fsa)?;
            let factored_determinized_fsa: VectorFst<GallicWeightRestrict<W>> =
                factor_weight::<
                    _,
                    VectorFst<GallicWeightRestrict<W>>,
                    _,
                    _,
                    GallicFactorRestrict<W>,
                >(&determinized_fsa, factor_opts)?;
            weight_convert(&factored_determinized_fsa, &mut from_gallic)
        }
        DeterminizeType::DeterminizeNonFunctional => {
            let fsa: VectorFst<GallicWeight<W>> = weight_convert(fst_in, &mut to_gallic)?;
            let determinized_fsa: VectorFst<GallicWeight<W>> =
                determinize_fsa::<_, _, _, GallicCommonDivisor>(&fsa)?;
            let factored_determinized_fsa: VectorFst<GallicWeight<W>> =
                factor_weight::<_, VectorFst<GallicWeight<W>>, _, _, GallicFactor<W>>(
                    &determinized_fsa,
                    factor_opts,
                )?;
            weight_convert(&factored_determinized_fsa, &mut from_gallic)
        }
    }
}

/// This operations creates an equivalent FST that has the property that no
/// state has two transitions with the same input label. For this algorithm,
/// epsilon transitions are treated as regular symbols.
///
/// # Example
///
/// ## Input
///
/// ![determinize_in](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/determinize_in.svg?sanitize=true)
///
/// ## Determinize
///
/// ![determinize_out](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/determinize_out.svg?sanitize=true)
///
pub fn determinize<W, F1, F2>(fst_in: &F1, det_type: DeterminizeType) -> Result<F2>
where
    W: WeaklyDivisibleSemiring + WeightQuantize + 'static,
    F1: ExpandedFst<W>,
    F2: MutableFst<W> + AllocableFst<W>,
{
    let mut fst_res: F2 = if fst_in.is_acceptor() {
        determinize_fsa::<_, _, _, DefaultCommonDivisor>(fst_in)?
    } else {
        determinize_fst(fst_in, det_type)?
    };

    fst_res.set_symts_from_fst(fst_in);
    Ok(fst_res)
}

#[cfg(test)]
mod tests {
    use crate::fst_impls::VectorFst;
    use crate::semirings::TropicalWeight;
    use crate::tr::Tr;

    use super::*;

    #[test]
    fn test_determinize() -> Result<()> {
        let mut input_fst = VectorFst::<TropicalWeight>::new();
        let s0 = input_fst.add_state();
        let s1 = input_fst.add_state();

        input_fst.set_start(s0)?;
        input_fst.set_final(s1, TropicalWeight::one())?;

        input_fst.add_tr(s0, Tr::new(1, 1, 2.0, s1))?;
        input_fst.add_tr(s0, Tr::new(1, 1, 2.0, s1))?;
        input_fst.add_tr(s0, Tr::new(1, 1, 2.0, s1))?;

        let mut ref_fst = VectorFst::new();
        let s0 = ref_fst.add_state();
        let s1 = ref_fst.add_state();

        ref_fst.set_start(s0)?;
        ref_fst.set_final(s1, TropicalWeight::one())?;

        ref_fst.add_tr(s0, Tr::new(1, 1, TropicalWeight::new(2.0), s1))?;

        let determinized_fst: VectorFst<TropicalWeight> =
            determinize(&ref_fst, DeterminizeType::DeterminizeFunctional)?;

        assert_eq!(determinized_fst, ref_fst);
        Ok(())
    }

    #[test]
    fn test_determinize_2() -> Result<()> {
        let mut input_fst = VectorFst::<TropicalWeight>::new();
        let s0 = input_fst.add_state();
        let s1 = input_fst.add_state();
        let s2 = input_fst.add_state();
        let s3 = input_fst.add_state();

        input_fst.set_start(s0)?;
        input_fst.set_final(s3, TropicalWeight::one())?;

        input_fst.add_tr(s0, Tr::new(1, 1, 2.0, s1))?;
        input_fst.add_tr(s0, Tr::new(1, 1, 3.0, s2))?;

        input_fst.add_tr(s1, Tr::new(2, 2, 4.0, s3))?;
        input_fst.add_tr(s2, Tr::new(2, 2, 3.0, s3))?;

        let mut ref_fst = VectorFst::new();
        let s0 = ref_fst.add_state();
        let s1 = ref_fst.add_state();
        let s2 = ref_fst.add_state();

        ref_fst.set_start(s0)?;
        ref_fst.set_final(s2, TropicalWeight::one())?;

        ref_fst.add_tr(s0, Tr::new(1, 1, TropicalWeight::new(2.0), s1))?;
        ref_fst.add_tr(s1, Tr::new(2, 2, TropicalWeight::new(4.0), s2))?;

        let determinized_fst: VectorFst<TropicalWeight> =
            determinize(&ref_fst, DeterminizeType::DeterminizeFunctional)?;

        assert_eq!(determinized_fst, ref_fst);
        Ok(())
    }
}
