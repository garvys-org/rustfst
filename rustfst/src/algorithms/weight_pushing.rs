use failure::Fallible;

use crate::algorithms::{reverse, reweight, shortest_distance, ReweightType};
use crate::fst_impls::VectorFst;
use crate::fst_traits::{ExpandedFst, Fst, MutableFst, CoreFst};
use crate::semirings::WeaklyDivisibleSemiring;
use crate::semirings::Semiring;

/// Pushes the weights in FST in the direction defined by TYPE. If
/// pushing towards the initial state, the sum of the weight of the
/// outgoing transitions and final weight at a non-initial state is
/// equal to One() in the resulting machine. If pushing towards the
/// final state, the same property holds on the reverse machine.
pub fn push_weights<F>(fst: &mut F, reweight_type: ReweightType) -> Fallible<()>
where
    F: Fst + ExpandedFst + MutableFst,
    F::W: WeaklyDivisibleSemiring,
    <<F as CoreFst>::W as Semiring>::ReverseWeight: 'static
{
    let dist = shortest_distance(fst, reweight_type == ReweightType::ReweightToInitial)?;
    reweight(fst, &dist, reweight_type)?;
    Ok(())
}
