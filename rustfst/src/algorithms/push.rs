use anyhow::Result;

use bitflags::bitflags;

use crate::algorithms::factor_weight::factor_iterators::{GallicFactorLeft, GallicFactorRight};
use crate::algorithms::factor_weight::{factor_weight, FactorWeightOptions, FactorWeightType};
use crate::algorithms::fst_convert::fst_convert_from_ref;
use crate::algorithms::tr_mappers::RmWeightMapper;
use crate::algorithms::weight_converters::{FromGallicConverter, ToGallicConverter};
use crate::algorithms::{
    reweight, shortest_distance_with_config, tr_map, weight_convert, ReweightType,
    ShortestDistanceConfig,
};
use crate::fst_impls::VectorFst;
use crate::fst_traits::{AllocableFst, ExpandedFst, MutableFst};
use crate::semirings::{DivideType, Semiring};
use crate::semirings::{
    GallicWeightLeft, GallicWeightRight, StringWeightLeft, StringWeightRight,
    WeaklyDivisibleSemiring, WeightQuantize,
};
use crate::{StateId, KDELTA};

bitflags! {
    /// Configuration to control the behaviour of the pushing algorithm.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct PushType: u32 {
        const PUSH_WEIGHTS = 0b01;
        const PUSH_LABELS = 0b10;
        const REMOVE_TOTAL_WEIGHT = 0b100;
        const REMOVE_COMMON_AFFIX = 0b1000;
    }
}

/// Configuration for [`push_weights_with_config`].
#[derive(Clone, Debug, Copy, PartialOrd, PartialEq)]
pub struct PushWeightsConfig {
    delta: f32,
    remove_total_weight: bool,
}

impl Default for PushWeightsConfig {
    fn default() -> Self {
        Self {
            delta: KDELTA,
            remove_total_weight: false,
        }
    }
}

impl PushWeightsConfig {
    pub fn new(delta: f32, remove_total_weight: bool) -> Self {
        Self {
            delta,
            remove_total_weight,
        }
    }

    pub fn with_delta(self, delta: f32) -> Self {
        Self { delta, ..self }
    }

    pub fn with_remove_total_weight(self, remove_total_weight: bool) -> Self {
        Self {
            remove_total_weight,
            ..self
        }
    }
}

/// Push the weights in an FST.
///
/// If pushing towards the initial state, the sum of the weight of the
/// outgoing transitions and final weight at a non-initial state is
/// equal to One() in the resulting machine. If pushing towards the
/// final state, the same property holds on the reverse machine.
pub fn push_weights<W, F>(fst: &mut F, reweight_type: ReweightType) -> Result<()>
where
    F: MutableFst<W>,
    W: WeaklyDivisibleSemiring,
{
    push_weights_with_config(fst, reweight_type, PushWeightsConfig::default())
}

/// Push the weights in an FST, optionally removing the total weight.
///
/// If pushing towards the initial state, the sum of the weight of the
/// outgoing transitions and final weight at a non-initial state is
/// equal to One() in the resulting machine. If pushing towards the
/// final state, the same property holds on the reverse machine.
pub fn push_weights_with_config<W, F>(
    fst: &mut F,
    reweight_type: ReweightType,
    config: PushWeightsConfig,
) -> Result<()>
where
    F: MutableFst<W>,
    W: WeaklyDivisibleSemiring,
{
    let remove_total_weight = config.remove_total_weight;
    let delta = config.delta;
    let dist = shortest_distance_with_config(
        fst,
        reweight_type == ReweightType::ReweightToInitial,
        ShortestDistanceConfig::new(delta),
    )?;

    if remove_total_weight {
        let total_weight =
            compute_total_weight(fst, &dist, reweight_type == ReweightType::ReweightToInitial)?;
        reweight(fst, &dist, reweight_type)?;
        remove_weight(
            fst,
            total_weight,
            reweight_type == ReweightType::ReweightToFinal,
        )?;
    } else {
        reweight(fst, &dist, reweight_type)?;
    }
    Ok(())
}

fn compute_total_weight<W, F>(fst: &F, dist: &[W], reverse: bool) -> Result<W>
where
    W: Semiring,
    F: ExpandedFst<W>,
{
    if reverse {
        Ok(fst
            .start()
            .and_then(|start| dist.get(start as usize))
            .cloned()
            .unwrap_or_else(W::zero))
    } else {
        let mut sum = W::zero();
        for (s, dist_s) in dist.iter().enumerate() {
            sum.plus_assign(dist_s.times(
                unsafe { fst.final_weight_unchecked(s as StateId) }.unwrap_or_else(W::zero),
            )?)?;
        }
        Ok(sum)
    }
}

fn remove_weight<W, F>(fst: &mut F, weight: W, at_final: bool) -> Result<()>
where
    F: MutableFst<W>,
    W: WeaklyDivisibleSemiring,
{
    if weight.is_one() || weight.is_zero() {
        return Ok(());
    }
    if at_final {
        unsafe {
            for s in fst.states_range() {
                if let Some(mut final_weight) = fst.final_weight_unchecked(s) {
                    final_weight.divide_assign(&weight, DivideType::DivideRight)?;
                    fst.set_final_unchecked(s, final_weight);
                }
            }
        }
    } else if let Some(start) = fst.start() {
        unsafe {
            let mut it_tr = fst.tr_iter_unchecked_mut(start);
            for idx_tr in 0..it_tr.len() {
                let tr = it_tr.get_unchecked(idx_tr);
                let weight = tr.weight.divide(&weight, DivideType::DivideLeft)?;
                it_tr.set_weight_unchecked(idx_tr, weight);
            }
            if let Some(mut final_weight) = fst.final_weight_unchecked(start) {
                final_weight.divide_assign(&weight, DivideType::DivideLeft)?;
                fst.set_final_unchecked(start, final_weight);
            }
        }
    }
    Ok(())
}

macro_rules! m_labels_pushing {
    ($ifst: ident, $reweight_type: ident, $push_type: ident, $delta: ident, $gallic_weight: ty, $string_weight: ident, $gallic_factor: ty) => {{
        // Labels pushing with potentially weights pushing
        let mut mapper = ToGallicConverter {};
        let mut gfst: VectorFst<$gallic_weight> = weight_convert($ifst, &mut mapper)?;
        let gdistance = if $push_type.intersects(PushType::PUSH_WEIGHTS) {
            shortest_distance_with_config(
                &gfst,
                $reweight_type == ReweightType::ReweightToInitial,
                ShortestDistanceConfig::new($delta),
            )?
        } else {
            let rm_weight_mapper = RmWeightMapper {};
            let mut uwfst: VectorFst<_> = fst_convert_from_ref($ifst);
            tr_map(&mut uwfst, &rm_weight_mapper)?;
            let guwfst: VectorFst<$gallic_weight> = weight_convert(&uwfst, &mut mapper)?;
            shortest_distance_with_config(
                &guwfst,
                $reweight_type == ReweightType::ReweightToInitial,
                ShortestDistanceConfig::new($delta),
            )?
        };
        if $push_type.intersects(PushType::REMOVE_COMMON_AFFIX | PushType::REMOVE_TOTAL_WEIGHT) {
            let mut total_weight = compute_total_weight(
                &gfst,
                &gdistance,
                $reweight_type == ReweightType::ReweightToInitial,
            )?;
            if !$push_type.intersects(PushType::REMOVE_COMMON_AFFIX) {
                total_weight.set_value1($string_weight::one());
            }
            if !$push_type.intersects(PushType::REMOVE_TOTAL_WEIGHT) {
                total_weight.set_value2(W::one());
            }
            reweight(&mut gfst, gdistance.as_slice(), $reweight_type)?;
            remove_weight(
                &mut gfst,
                total_weight,
                $reweight_type == ReweightType::ReweightToFinal,
            )?;
        } else {
            reweight(&mut gfst, gdistance.as_slice(), $reweight_type)?;
        }
        let fwfst: VectorFst<$gallic_weight> =
            factor_weight::<_, VectorFst<$gallic_weight>, _, _, $gallic_factor>(
                &gfst,
                FactorWeightOptions::new(
                    FactorWeightType::FACTOR_FINAL_WEIGHTS | FactorWeightType::FACTOR_ARC_WEIGHTS,
                ),
            )?;
        let mut mapper_from_gallic = FromGallicConverter {
            superfinal_label: 0,
        };
        weight_convert(&fwfst, &mut mapper_from_gallic)
    }};
}

/// Configuration for [`push_with_config`].
#[derive(Clone, Copy, Debug, PartialOrd, PartialEq)]
pub struct PushConfig {
    delta: f32,
}

impl Default for PushConfig {
    fn default() -> Self {
        Self { delta: KDELTA }
    }
}

impl PushConfig {
    pub fn new(delta: f32) -> Self {
        Self { delta }
    }

    pub fn with_delta(self, delta: f32) -> Self {
        Self { delta }
    }
}

/// Push the weights and/or labels of the input FST into the output
/// mutable FST by pushing weights and/or labels towards the initial state or final states.
pub fn push<W, F1, F2>(ifst: &F1, reweight_type: ReweightType, push_type: PushType) -> Result<F2>
where
    F1: ExpandedFst<W>,
    F2: ExpandedFst<W> + MutableFst<W> + AllocableFst<W>,
    W: WeaklyDivisibleSemiring + WeightQuantize,
    <W as Semiring>::ReverseWeight: 'static,
{
    push_with_config(ifst, reweight_type, push_type, PushConfig::default())
}

/// Push the weights and/or labels of the input FST into the output
/// mutable FST by pushing weights and/or labels towards the initial state or final states.
pub fn push_with_config<W, F1, F2>(
    ifst: &F1,
    reweight_type: ReweightType,
    push_type: PushType,
    config: PushConfig,
) -> Result<F2>
where
    F1: ExpandedFst<W>,
    F2: ExpandedFst<W> + MutableFst<W> + AllocableFst<W>,
    W: WeaklyDivisibleSemiring + WeightQuantize,
    <W as Semiring>::ReverseWeight: 'static,
{
    let delta = config.delta;
    if push_type.intersects(PushType::PUSH_WEIGHTS) && !push_type.intersects(PushType::PUSH_LABELS)
    {
        // Only weights pushing
        let mut ofst = fst_convert_from_ref(ifst);
        let push_weights_config =
            PushWeightsConfig::new(delta, push_type.intersects(PushType::REMOVE_TOTAL_WEIGHT));
        push_weights_with_config(&mut ofst, reweight_type, push_weights_config)?;
        Ok(ofst)
    } else if push_type.intersects(PushType::PUSH_LABELS) {
        match reweight_type {
            ReweightType::ReweightToInitial => m_labels_pushing!(
                ifst,
                reweight_type,
                push_type,
                delta,
                GallicWeightLeft<W>,
                StringWeightLeft,
                GallicFactorLeft<W>
            ),
            ReweightType::ReweightToFinal => m_labels_pushing!(
                ifst,
                reweight_type,
                push_type,
                delta,
                GallicWeightRight<W>,
                StringWeightRight,
                GallicFactorRight<W>
            ),
        }
    } else {
        // NO Labels/Weights pushing
        Ok(fst_convert_from_ref(ifst))
    }
}
