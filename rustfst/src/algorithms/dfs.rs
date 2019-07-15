use std::collections::HashSet;
use std::hash::BuildHasher;

use failure::Fallible;

use crate::fst_traits::Fst;
use crate::StateId;

#[deprecated]
pub fn dfs<F: Fst, S1: BuildHasher, S2: BuildHasher>(
    fst: &F,
    state_id_cour: StateId,
    accessible_states: &mut HashSet<StateId, S1>,
    coaccessible_states: &mut HashSet<StateId, S2>,
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
    state_id_to_lol: Vec<Option<TempSccs>>,
    fst: &'a F,
    next_scc_id: usize,
}

#[derive(Clone)]
struct TempSccs {
    index: StateId,
    lowlink: StateId,
    on_stack: bool,
    scc: Option<usize>,
}

impl<'a, F: Fst> TarjanAlgorithm<'a, F> {
    fn new(fst: &'a F) -> Self {
        Self {
            index: 0,
            stack: vec![],
            state_id_to_lol: vec![],
            fst,
            next_scc_id: 0,
        }
    }

    fn compute(&mut self) -> Fallible<()> {
        let states: Vec<_> = self.fst.states_iter().collect();
        self.state_id_to_lol.resize(states.len(), None);
        for state in states {
            if self.state_id_to_lol[state].is_none() {
                self.strongconnect(state)?;
            }
        }
        Ok(())
    }

    fn strongconnect(&mut self, state: StateId) -> Fallible<()> {
        let lol = TempSccs {
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
                let w_lowlink = self.state_id_to_lol[arc.nextstate]
                    .as_ref()
                    .unwrap()
                    .lowlink;
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
                    break;
                }
            }
            self.next_scc_id += 1;
        }

        Ok(())
    }

    fn get_scc(self) -> Vec<usize> {
        self.state_id_to_lol
            .into_iter()
            .map(|o_lol| o_lol.unwrap().scc.unwrap())
            .collect()
    }
}

pub fn find_strongly_connected_components<F: Fst>(
    fst: &F,
    sccs: &mut Vec<usize>,
    n_sccs: &mut usize,
) -> Fallible<()> {
    let mut tarjan = TarjanAlgorithm::new(fst);
    tarjan.compute()?;
    *n_sccs = tarjan.next_scc_id;
    *sccs = tarjan.get_scc();
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::fst_impls::VectorFst;
    use crate::fst_traits::MutableFst;
    use crate::semirings::Semiring;
    use crate::semirings::TropicalWeight;
    use crate::Arc;

    #[test]
    fn test_sccs_1() -> Fallible<()> {
        let mut fst = VectorFst::new();
        let s0 = fst.add_state();
        let s1 = fst.add_state();
        let s2 = fst.add_state();

        fst.set_start(s0)?;
        fst.set_final(s1, TropicalWeight::one())?;

        fst.add_arc(s0, Arc::new(1, 1, TropicalWeight::one(), s1))?;
        fst.add_arc(s1, Arc::new(2, 2, TropicalWeight::one(), s2))?;

        // Every state has its own scc.
        let mut sccs = vec![];
        let mut n_sccs = 0;
        find_strongly_connected_components(&fst, &mut sccs, &mut n_sccs)?;
        assert_eq!(n_sccs, 3);
        assert_eq!(sccs[s0], 2);
        assert_eq!(sccs[s1], 1);
        assert_eq!(sccs[s2], 0);

        Ok(())
    }

    #[test]
    fn test_sccs_2() -> Fallible<()> {
        let mut fst = VectorFst::new();
        let s0 = fst.add_state();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        let s3 = fst.add_state();
        let s4 = fst.add_state();

        fst.add_arc(s0, Arc::new(1, 1, TropicalWeight::one(), s2))?;
        fst.add_arc(s2, Arc::new(1, 1, TropicalWeight::one(), s1))?;
        fst.add_arc(s1, Arc::new(1, 1, TropicalWeight::one(), s0))?;
        fst.add_arc(s0, Arc::new(2, 2, TropicalWeight::one(), s3))?;
        fst.add_arc(s3, Arc::new(1, 1, TropicalWeight::one(), s4))?;

        // Every state has its own scc.
        let mut sccs = vec![];
        let mut n_sccs = 0;
        find_strongly_connected_components(&fst, &mut sccs, &mut n_sccs)?;
        assert_eq!(n_sccs, 3);
        assert_eq!(sccs[s0], 2);
        assert_eq!(sccs[s1], 2);
        assert_eq!(sccs[s2], 2);
        assert_eq!(sccs[s3], 1);
        assert_eq!(sccs[s4], 0);

        Ok(())
    }
}
