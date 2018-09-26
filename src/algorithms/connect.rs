use fst_traits::{ExpandedFst, Fst, MutableFst};
use std::collections::HashSet;
use StateId;

fn dfs<F: Fst>(
    fst: &F,
    state_id_cour: StateId,
    accessible_states: &mut HashSet<StateId>,
    coaccessible_states: &mut HashSet<StateId>,
) {
    accessible_states.insert(state_id_cour);
    let mut is_coaccessible = fst.is_final(&state_id_cour);
    for arc in fst.arcs_iter(&state_id_cour) {
        let nextstate = arc.nextstate;

        if !accessible_states.contains(&nextstate) {
            dfs(fst, nextstate, accessible_states, coaccessible_states);
        }

        if coaccessible_states.contains(&nextstate) {
            is_coaccessible = true;
        }
    }

    if is_coaccessible {
        coaccessible_states.insert(state_id_cour);
    }
}

pub fn connect<F: ExpandedFst + MutableFst>(fst: &mut F) {
    let mut accessible_states = HashSet::new();
    let mut coaccessible_states = HashSet::new();

    if let Some(state_id) = fst.start() {
        dfs(
            fst,
            state_id,
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

#[cfg(test)]
mod tests {
    use super::*;
    use semirings::ProbabilityWeight;
    use fst_impls::VectorFst;

    #[test]
    fn test_connect() {
        let mut fst = VectorFst::new();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        fst.set_start(&s1);
        fst.add_arc(&s1, &s2, 3, 5, ProbabilityWeight::new(10.0));
        fst.add_arc(&s1, &s2, 5, 7, ProbabilityWeight::new(18.0));
        fst.set_final(&s2, ProbabilityWeight::new(31.0));
        fst.add_state();
        let s4 = fst.add_state();
        fst.add_arc(&s2, &s4, 5, 7, ProbabilityWeight::new(18.0));
        assert_eq!(fst.num_states(), 4);
        connect(&mut fst);
        assert_eq!(fst.num_states(), 2);
    }
}
