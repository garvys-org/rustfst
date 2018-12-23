use crate::arc::Arc;
use crate::fst_traits::{ExpandedFst, FinalStatesIterator, MutableFst};
use crate::semirings::Semiring;
use crate::{Result, EPS_LABEL};

pub fn reverse<W, F1, F2>(fst: &F1) -> Result<F2>
where
    W: Semiring,
    F1: ExpandedFst<W = W>,
    F2: MutableFst<W = W> + ExpandedFst<W = W>,
{
    let mut fst_reversed = F2::new();

    let num_states = fst.num_states();

    (0..num_states).for_each(|_| {
        fst_reversed.add_state();
    });

    // Reverse all the transitions
    for state in 0..num_states {
        for arc in fst.arcs_iter(state)? {
            fst_reversed.add_arc(
                arc.nextstate,
                Arc::new(arc.ilabel, arc.olabel, arc.weight.clone(), state),
            )?;
        }
    }

    // Creates the initial state
    let super_initial_state = fst_reversed.add_state();
    fst_reversed.set_start(super_initial_state)?;

    // Add epsilon arc from the initial state to the former final states
    for final_state in fst.final_states_iter() {
        fst_reversed.add_arc(
            super_initial_state,
            Arc::new(
                EPS_LABEL,
                EPS_LABEL,
                final_state.final_weight,
                final_state.state_id,
            ),
        )?;
    }

    // Forme initial states are now final
    if let Some(state_state_in) = fst.start() {
        fst_reversed.set_final(state_state_in, W::ONE)?;
    }

    Ok(fst_reversed)
}
