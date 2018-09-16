use fst::Fst;
use fst::{ExpandedFst, MutableFst};
use semirings::Semiring;
use std::collections::HashSet;
use StateId;

fn dfs<W: Semiring, F: Fst<W>>(
    fst: &F,
    state_id_cour: &StateId,
    accessible_states: &mut HashSet<StateId>,
    coaccessible_states: &mut HashSet<StateId>,
) {
    accessible_states.insert(*state_id_cour);
    let mut is_coaccessible = fst.is_final(state_id_cour);
    for arc in fst.arc_iter(state_id_cour) {
        let nextstate = arc.nextstate;

        if !accessible_states.contains(&nextstate) {
            dfs(fst, &nextstate, accessible_states, coaccessible_states);
        }

        if coaccessible_states.contains(&nextstate) {
            is_coaccessible = true;
        }
    }

    if is_coaccessible {
        coaccessible_states.insert(*state_id_cour);
    }
}

pub fn connect<W: Semiring, F: ExpandedFst<W> + MutableFst<W>>(fst: &mut F) {
    let mut accessible_states = HashSet::new();
    let mut coaccessible_states = HashSet::new();

    if let Some(state_id) = fst.start() {
        dfs(
            fst,
            &state_id,
            &mut accessible_states,
            &mut coaccessible_states,
        );
    }

    let mut to_delete = Vec::new();
    for i in 0..fst.num_states() {
        if !accessible_states.contains(&i) || !coaccessible_states.contains(&i) {
            to_delete.push(i);
        }
    }
    fst.del_states(to_delete);
}
