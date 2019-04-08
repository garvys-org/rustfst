use std::collections::HashSet;

use failure::Fallible;

use crate::algorithms::state_sort;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{ExpandedFst, Fst, MutableFst};
use crate::StateId;

fn dfs_topsort<F: Fst>(
    fst: &F,
    state_id_cour: StateId,
    accessible_states: &mut HashSet<StateId>,
    order: &mut Vec<StateId>,
    cyclic: &mut bool,
) -> Fallible<()> {
    //    order[state_id_cour] = Some(*idx);
    accessible_states.insert(state_id_cour);

    for arc in fst.arcs_iter(state_id_cour)? {
        let nextstate = arc.nextstate;

        if !accessible_states.contains(&nextstate) {
            //            *cyclic = true;
            dfs_topsort(fst, nextstate, accessible_states, order, cyclic)?;
        }
    }

    order.push(state_id_cour);

    Ok(())
}

pub fn top_sort<F>(fst: &mut F) -> Fallible<()>
where
    F: MutableFst + ExpandedFst,
{
    let mut accessible_states = HashSet::new();
    let mut finish = vec![];
    let mut cyclic = false;
    if let Some(start) = fst.start() {
        dfs_topsort(fst, start, &mut accessible_states, &mut finish, &mut cyclic)?;
    }
    // Topsort unreachable state.
    for state in fst.states_iter() {
        if !accessible_states.contains(&state) {
            dfs_topsort(fst, state, &mut accessible_states, &mut finish, &mut cyclic)?;
        }
    }

    let acyclic = fst.properties()?.contains(FstProperties::ACYCLIC);
    if acyclic {
        let mut order: Vec<StateId> = vec![0; finish.len()];
        let finish_len = finish.len();
        for s in 0..finish_len {
            order[finish[finish_len - s - 1]] = s;
        }
        state_sort(fst, &order)?;
    }

    Ok(())
}
