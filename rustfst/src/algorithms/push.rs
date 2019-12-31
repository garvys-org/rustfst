use failure::Fallible;

use bitflags::bitflags;

use crate::algorithms::arc_mappers::RmWeightMapper;
use crate::algorithms::factor_iterators::{GallicFactorLeft, GallicFactorRight};
use crate::algorithms::fst_convert::fst_convert;
use crate::algorithms::weight_converters::{FromGallicConverter, ToGallicConverter};
use crate::algorithms::{
    arc_map, factor_weight, reweight, shortest_distance, weight_convert, FactorWeightOptions,
    FactorWeightType, ReweightType,
};
use crate::fst_impls::VectorFst;
use crate::fst_traits::{CoreFst, ExpandedFst, Fst, MutableFst, AllocableFst};
use crate::semirings::{DivideType, Semiring};
use crate::semirings::{
    GallicWeightLeft, GallicWeightRight, StringWeightLeft, StringWeightRight,
    WeaklyDivisibleSemiring, WeightQuantize,
};

bitflags! {
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
pub fn push_weights<F>(
    fst: &mut F,
    reweight_type: ReweightType,
    remove_total_weight: bool,
) -> Fallible<()>
where
    F: Fst + ExpandedFst + MutableFst,
    F::W: WeaklyDivisibleSemiring,
    <<F as CoreFst>::W as Semiring>::ReverseWeight: 'static,
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

fn compute_total_weight<F>(fst: &F, dist: &Vec<F::W>, reverse: bool) -> Fallible<F::W>
where
    F: ExpandedFst,
{
    if reverse {
        if let Some(start) = fst.start() {
            if start < dist.len() {
                Ok(dist[start].clone())
            } else {
                Ok(F::W::zero())
            }
        } else {
            Ok(F::W::zero())
        }
    } else {
        let mut sum = F::W::zero();
        let zero = F::W::zero();
        for s in 0..dist.len() {
            sum.plus_assign(
                dist[s].times(unsafe { fst.final_weight_unchecked(s) }.unwrap_or_else(|| &zero))?,
            )?;
        }
        Ok(sum)
    }
}

fn remove_weight<F>(fst: &mut F, weight: F::W, at_final: bool) -> Fallible<()>
where
    F: MutableFst + ExpandedFst,
    F::W: WeaklyDivisibleSemiring,
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
    } else {
        if let Some(start) = fst.start() {
            for arc in unsafe { fst.arcs_iter_unchecked_mut(start) } {
                arc.weight.divide_assign(&weight, DivideType::DivideLeft)?;
            }
            if let Some(final_weight) = unsafe { fst.final_weight_unchecked_mut(start) } {
                final_weight.divide_assign(&weight, DivideType::DivideLeft)?;
            }
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
            let mut uwfst: VectorFst<_> = fst_convert($ifst);
            arc_map(&mut uwfst, &mut rm_weight_mapper)?;
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
                total_weight.set_value2(F1::W::one());
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
        let fwfst: VectorFst<_> = factor_weight::<_, _, $gallic_factor>(
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

pub fn push<F1, F2>(ifst: &F1, reweight_type: ReweightType, push_type: PushType) -> Fallible<F2>
where
    F1: ExpandedFst,
    F1::W: WeaklyDivisibleSemiring + WeightQuantize,
    F2: ExpandedFst<W = F1::W> + MutableFst + AllocableFst,
    <F1 as CoreFst>::W: 'static,
    <<F1 as CoreFst>::W as Semiring>::ReverseWeight: 'static,
{
    if push_type.intersects(PushType::PUSH_WEIGHTS) && !push_type.intersects(PushType::PUSH_LABELS)
    {
        // Only weights pushing
        let mut ofst = fst_convert(ifst);
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
                GallicWeightLeft<F1::W>,
                StringWeightLeft,
                GallicFactorLeft<F1::W>
            ),
            ReweightType::ReweightToFinal => m_labels_pushing!(
                ifst,
                reweight_type,
                push_type,
                GallicWeightRight<F1::W>,
                StringWeightRight,
                GallicFactorRight<F1::W>
            ),
        }
    } else {
        // NO Labels/Weights pushing
        Ok(fst_convert(ifst))
    }
}
