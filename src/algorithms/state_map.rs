use failure::Fallible;

use crate::fst_traits::{Fst, MutableFst};
use crate::semirings::Semiring;
use crate::StateId;

pub trait StateMapper<F: MutableFst> {
    fn map_final_weight(&self, weight: Option<&mut F::W>);
    fn map_arcs(&self, fst: &mut F, state: StateId);
}

/// This operation transforms each state in the input FST.
/// The transformation is specified by a function object called a state mapper.
///
/// For instance, ArcSumMapper doc combines arcs with the same input label,
/// output label and destination state, ⊕-summing their weights.
pub fn state_map<F, M>(ifst: &mut F, mapper: &mut M) -> Fallible<()>
where
    F: MutableFst,
    M: StateMapper<F>,
{
    if ifst.start().is_none() {
        return Ok(());
    }

    let states: Vec<_> = ifst.states_iter().collect();

    for state in states {
        mapper.map_arcs(ifst, state);
        mapper.map_final_weight(ifst.final_weight_mut(state));
    }

    Ok(())
}
