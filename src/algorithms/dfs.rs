use std::collections::HashSet;

use failure::Fallible;

use crate::fst_traits::Fst;
use crate::StateId;


pub fn dfs<F: Fst>(
    fst: &F,
    state_id_cour: StateId,
    accessible_states: &mut HashSet<StateId>,
    coaccessible_states: &mut HashSet<StateId>,
) -> Fallible<()> {
    accessible_states.insert(state_id_cour);
    let mut is_coaccessible = fst.is_final(state_id_cour);
    for arc in fst.arcs_iter(state_id_cour)? {
        let nextstate = arc.nextstate;

        if !accessible_states.contains(&nextstate) {
            dfs(fst, nextstate, accessible_states, coaccessible_states)?;
        }

        if coaccessible_states.contains(&nextstate) {
            is_coaccessible = true;
        }
    }

    if is_coaccessible {
        coaccessible_states.insert(state_id_cour);
    }

    Ok(())
}