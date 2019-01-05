use crate::fst_traits::{ExpandedFst, Fst, MutableFst};
use crate::semirings::WeaklyDivisibleSemiring;
use crate::Result;
use crate::{reverse, reweight, shortest_distance, ReweightType};

/// Pushes the weights in FST in the direction defined by TYPE. If
/// pushing towards the initial state, the sum of the weight of the
/// outgoing transitions and final weight at a non-initial state is
/// equal to One() in the resulting machine. If pushing towards the
/// final state, the same property holds on the reverse machine.
pub fn push_weights<F>(fst: &mut F, reweight_type: ReweightType) -> Result<()>
where
    F: Fst + ExpandedFst + MutableFst,
    F::W: WeaklyDivisibleSemiring,
{
    match reweight_type {
        ReweightType::ReweightToInitial => {
            let fst_reversed: F = reverse(fst)?;
            let dist = shortest_distance(&fst_reversed)?;

            reweight(fst, &dist, ReweightType::ReweightToInitial)
        }
        ReweightType::ReweightToFinal => {
            let dist = shortest_distance(fst)?;

            reweight(fst, &dist, ReweightType::ReweightToFinal)
        }
    }
}
