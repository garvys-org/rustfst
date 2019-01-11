use crate::arc::Arc;
use crate::fst_traits::{ExpandedFst, FinalStatesIterator, MutableFst};
use crate::semirings::Semiring;
use crate::{Result, EPS_LABEL};

/// Reverses an FST. The reversed result is written to an output mutable FST.
/// If A transduces string x to y with weight a, then the reverse of A
/// transduces the reverse of x to the reverse of y with weight a.Reverse().
///
/// Typically, a = a.Reverse() and an arc is its own reverse (e.g., for
/// TropicalWeight or LogWeight). In general, e.g., when the weights only form a
/// left or right semiring, the output arc type must match the input arc type
/// except having the reversed Weight type.
///
/// A superinitial state is always created.
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
        fst_reversed.set_final(state_state_in, W::one())?;
    }

    Ok(fst_reversed)
}
