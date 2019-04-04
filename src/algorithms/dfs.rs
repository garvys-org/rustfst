use std::collections::HashSet;

use failure::Fallible;

use crate::fst_traits::{Fst, ExpandedFst};
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


struct TarjanAlgorithm<'a, F: Fst> {
    index: usize,
    stack: Vec<StateId>,
    state_id_to_lol: Vec<Option<Lol>>,
    fst: &'a F,
    next_scc_id: usize
}

impl<'a, F: Fst> TarjanAlgorithm<'a, F> {
    pub fn new(fst: &'a F) -> Self {
        Self {
            index: 0,
            stack: vec![],
            state_id_to_lol: vec![],
            fst,
            next_scc_id: 0
        }
    }

    pub fn compute(&mut self) -> Fallible<()> {
        let states : Vec<_> = self.fst.states_iter().collect();
        self.state_id_to_lol.resize(states.len(), None);
        for state in states {
            if self.state_id_to_lol[state].is_none() {
                self.strongconnect(state)?;
            }
        }
        Ok(())
    }

    fn strongconnect(&mut self, state: StateId) -> Fallible<()> {

        let lol = Lol {
            index: self.index,
            lowlink: self.index,
            on_stack: true,
            scc: None,
        };
        self.index += 1;
        self.stack.push(state);
        self.state_id_to_lol[state] = Some(lol);

        // Consider successors of v
        for arc in self.fst.arcs_iter(state)? {
            if self.state_id_to_lol[arc.nextstate].is_none() {
                self.strongconnect(arc.nextstate)?;
                let v_lowlink = self.state_id_to_lol[state].as_ref().unwrap().lowlink;
                let w_lowlink = self.state_id_to_lol[arc.nextstate].as_ref().unwrap().lowlink;
                self.state_id_to_lol[state].as_mut().unwrap().lowlink = v_lowlink.min(w_lowlink);
            } else {
                let w = self.state_id_to_lol[arc.nextstate].as_ref().unwrap();
                if w.on_stack {
                    // Successor w is in stack S and hence in the current SCC
                    // If w is not on stack, then (v, w) is a cross-edge in the DFS tree and must be ignored
                    // Note: The next line may look odd - but is correct.
                    // It says w.index not w.lowlink; that is deliberate and from the original paper]
                    let v_lowlink = self.state_id_to_lol[state].as_ref().unwrap().lowlink;
                    let w_index = self.state_id_to_lol[arc.nextstate].as_ref().unwrap().index;
                    self.state_id_to_lol[state].as_mut().unwrap().lowlink = v_lowlink.min(w_index);
                }
            }
        }
        let v = self.state_id_to_lol[state].as_ref().unwrap();
        if v.lowlink == v.index {
            loop {
                let w_state_id = self.stack.pop().unwrap();
                self.state_id_to_lol[w_state_id].as_mut().unwrap().on_stack = false;
                self.state_id_to_lol[w_state_id].as_mut().unwrap().scc = Some(self.next_scc_id);
                if w_state_id == state {
                    break
                }
            }
            self.next_scc_id += 1;
        }

        Ok(())
    }
}

#[derive(Clone)]
struct Lol {
    index: StateId,
    lowlink: StateId,
    on_stack: bool,
    scc: Option<usize>
}

