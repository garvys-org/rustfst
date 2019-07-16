use failure::Fallible;

use crate::algorithms::{reweight, shortest_distance, ReweightType};
use crate::fst_traits::{CoreFst, ExpandedFst, Fst, MutableFst};
use crate::semirings::WeaklyDivisibleSemiring;
use crate::semirings::{DivideType, Semiring};

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
            reweight_type == ReweightType::ReweightToInitial,
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
        for s in 0..dist.len() {
            sum.plus_assign(dist[s].times(fst.final_weight(s).unwrap_or_else(F::W::zero))?)?;
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
            if let Some(final_weight) = fst.final_weight_mut(s) {
                final_weight.divide(&weight, DivideType::DivideRight)?;
            }
        }
    } else {
        if let Some(start) = fst.start() {
            for arc in unsafe { fst.arcs_iter_unchecked_mut(start) } {
                arc.weight.divide_assign(&weight, DivideType::DivideLeft)?;
            }
            if let Some(final_weight) = fst.final_weight_mut(start) {
                final_weight.divide_assign(&weight, DivideType::DivideLeft)?;
            }
        }
    }
    Ok(())
}
