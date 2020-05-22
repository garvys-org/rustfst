use anyhow::Result;

use bitflags::bitflags;

use crate::algorithms::factor_weight::factor_iterators::{GallicFactorLeft, GallicFactorRight};
use crate::algorithms::factor_weight::{factor_weight, FactorWeightOptions, FactorWeightType};
use crate::algorithms::fst_convert::fst_convert_from_ref;
use crate::algorithms::tr_mappers::RmWeightMapper;
use crate::algorithms::weight_converters::{FromGallicConverter, ToGallicConverter};
use crate::algorithms::{reweight, shortest_distance, tr_map, weight_convert, ReweightType};
use crate::fst_impls::VectorFst;
use crate::fst_traits::{AllocableFst, ExpandedFst, MutableFst};
use crate::semirings::{DivideType, Semiring};
use crate::semirings::{
    GallicWeightLeft, GallicWeightRight, StringWeightLeft, StringWeightRight,
    WeaklyDivisibleSemiring, WeightQuantize,
};

bitflags! {
    /// Configuration to control the behaviour of the pushing algorithm.
    pub struct PushType: u32 {
        const PUSH_WEIGHTS = 0b01;
        const PUSH_LABELS = 0b10;
        const REMOVE_TOTAL_WEIGHT = 0b100;
        const REMOVE_COMMON_AFFIX = 0b1000;
    }
}

/// Pushes the weights in FST in the direction defined by TYPE. If
/// pushing towards the initial state, the sum of the weight of the
/// outgoing transitions and final weight at a non-initial state is
/// equal to One() in the resulting machine. If pushing towards the
/// final state, the same property holds on the reverse machine.
pub fn push_weights<W, F>(
    fst: &mut F,
    reweight_type: ReweightType,
    remove_total_weight: bool,
) -> Result<()>
where
    F: MutableFst<W>,
    W: WeaklyDivisibleSemiring,
{
    let dist = shortest_distance(fst, reweight_type == ReweightType::ReweightToInitial)?;
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
        if let Some(start) = fst.start() {
            if start < dist.len() {
                Ok(dist[start].clone())
            } else {
                Ok(W::zero())
            }
        } else {
            Ok(W::zero())
        }
    } else {
        let mut sum = W::zero();
        for s in 0..dist.len() {
            sum.plus_assign(
                dist[s].times(unsafe { fst.final_weight_unchecked(s) }.unwrap_or_else(W::zero))?,
            )?;
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
        for s in 0..fst.num_states() {
            if let Some(final_weight) = unsafe { fst.final_weight_unchecked_mut(s) } {
                final_weight.divide_assign(&weight, DivideType::DivideRight)?;
            }
        }
    } else if let Some(start) = fst.start() {
        for tr in unsafe { fst.tr_iter_unchecked_mut(start) } {
            tr.weight.divide_assign(&weight, DivideType::DivideLeft)?;
        }
        if let Some(final_weight) = unsafe { fst.final_weight_unchecked_mut(start) } {
            final_weight.divide_assign(&weight, DivideType::DivideLeft)?;
        }
    }
    Ok(())
}

macro_rules! m_labels_pushing {
    ($ifst: ident, $reweight_type: ident, $push_type: ident, $gallic_weight: ty, $string_weight: ident, $gallic_factor: ty) => {{
        // Labels pushing with potentially weights pushing
        let mut mapper = ToGallicConverter {};
        let mut gfst: VectorFst<$gallic_weight> = weight_convert($ifst, &mut mapper)?;
        let gdistance = if $push_type.intersects(PushType::PUSH_WEIGHTS) {
            shortest_distance(&gfst, $reweight_type == ReweightType::ReweightToInitial)?
        } else {
            let mut rm_weight_mapper = RmWeightMapper {};
            let mut uwfst: VectorFst<_> = fst_convert_from_ref($ifst);
            tr_map(&mut uwfst, &mut rm_weight_mapper)?;
            let guwfst: VectorFst<$gallic_weight> = weight_convert(&uwfst, &mut mapper)?;
            shortest_distance(&guwfst, $reweight_type == ReweightType::ReweightToInitial)?
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

/// Pushes the weights and/or labels of the input FST into the output
/// mutable FST by pushing weights and/or labels towards the initial state or final states.
pub fn push<W, F1, F2>(ifst: &F1, reweight_type: ReweightType, push_type: PushType) -> Result<F2>
where
    F1: ExpandedFst<W>,
    F2: ExpandedFst<W> + MutableFst<W> + AllocableFst<W>,
    W: 'static + WeaklyDivisibleSemiring + WeightQuantize,
    <W as Semiring>::ReverseWeight: 'static,
{
    if push_type.intersects(PushType::PUSH_WEIGHTS) && !push_type.intersects(PushType::PUSH_LABELS)
    {
        // Only weights pushing
        let mut ofst = fst_convert_from_ref(ifst);
        push_weights(
            &mut ofst,
            reweight_type,
            push_type.intersects(PushType::REMOVE_TOTAL_WEIGHT),
        )?;
        Ok(ofst)
    } else if push_type.intersects(PushType::PUSH_LABELS) {
        match reweight_type {
            ReweightType::ReweightToInitial => m_labels_pushing!(
                ifst,
                reweight_type,
                push_type,
                GallicWeightLeft<W>,
                StringWeightLeft,
                GallicFactorLeft<W>
            ),
            ReweightType::ReweightToFinal => m_labels_pushing!(
                ifst,
                reweight_type,
                push_type,
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
