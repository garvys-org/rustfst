use failure::Fallible;

use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::StateId;

/// StateMapper Interface. The class determines how states are mapped. This is useful for
/// implementing operations that do not change the number of states.
pub trait StateMapper<F: MutableFst> {
    /// Defines how final weight are mapped.
    fn map_final_weight(&self, weight: Option<&mut F::W>) -> Fallible<()>;
    /// Defines how arcs leaving the state `state` are mapped.
    fn map_arcs(&self, fst: &mut F, state: StateId) -> Fallible<()>;
}

/// This operation transforms each state in the input FST.
/// The transformation is specified by a function object called a `StateMapper`.
pub fn state_map<F, M>(ifst: &mut F, mapper: &mut M) -> Fallible<()>
where
    F: MutableFst + ExpandedFst,
    M: StateMapper<F>,
{
    if ifst.start().is_none() {
        return Ok(());
    }

    for state in 0..ifst.num_states() {
        mapper.map_arcs(ifst, state)?;
        mapper.map_final_weight(ifst.final_weight_mut(state))?;
    }

    Ok(())
}
