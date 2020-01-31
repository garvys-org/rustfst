use std::collections::hash_map::Entry;
use std::collections::HashMap;

use failure::Fallible;
use unsafe_unwrap::UnsafeUnwrap;

use crate::{EPS_LABEL, Label, StateId};
use crate::algorithms::arc_filters::{ArcFilter, EpsilonArcFilter};
use crate::algorithms::dfs_visit::dfs_visit;
use crate::algorithms::shortest_distance;
use crate::algorithms::top_sort::TopOrderVisitor;
use crate::algorithms::visitors::SccVisitor;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{Fst, MutableFst};
use crate::fst_traits::CoreFst;
use crate::semirings::Semiring;

/// This operation removes epsilon-transitions (when both the input and
/// output labels are an epsilon) from a transducer. The result will be an
/// equivalent FST that has no such epsilon transitions.
///
/// # Example
/// ```
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::fst_traits::MutableFst;
/// # use rustfst::algorithms::rm_epsilon;
/// # use rustfst::Arc;
/// # use rustfst::EPS_LABEL;
/// let mut fst = VectorFst::new();
/// let s0 = fst.add_state();
/// let s1 = fst.add_state();
/// fst.add_arc(s0, Arc::new(32, 25, IntegerWeight::new(78), s1));
/// fst.add_arc(s1, Arc::new(EPS_LABEL, EPS_LABEL, IntegerWeight::new(13), s0));
/// fst.set_start(s0).unwrap();
/// fst.set_final(s0, IntegerWeight::new(5));
///
/// let fst_no_epsilon : VectorFst<_> = rm_epsilon(&fst).unwrap();
///
/// let mut fst_no_epsilon_ref = VectorFst::<IntegerWeight>::new();
/// let s0 = fst_no_epsilon_ref.add_state();
/// let s1 = fst_no_epsilon_ref.add_state();
/// fst_no_epsilon_ref.add_arc(s0, Arc::new(32, 25, 78, s1));
/// fst_no_epsilon_ref.add_arc(s1, Arc::new(32, 25, 78 * 13, s1));
/// fst_no_epsilon_ref.set_start(s0).unwrap();
/// fst_no_epsilon_ref.set_final(s0, 5);
/// fst_no_epsilon_ref.set_final(s1, 5 * 13);
///
/// assert_eq!(fst_no_epsilon, fst_no_epsilon_ref);
/// ```
pub fn rm_epsilon<F: MutableFst>(fst: &mut F) -> Fallible<()>
where
    <<F as CoreFst>::W as Semiring>::ReverseWeight: 'static,
{
    let start_state = fst.start();
    if start_state.is_none() {
        return Ok(());
    }
    let start_state = unsafe { start_state.unsafe_unwrap() };

    // noneps_in[s] will be set to true iff s admits a non-epsilon incoming
    // transition or is the start state.
    let mut noneps_in = vec![false; fst.num_states()];
    noneps_in[start_state] = true;

    for state in 0..fst.num_states() {
        for arc in fst.arcs_iter(state)? {
            if arc.ilabel != EPS_LABEL || arc.olabel != EPS_LABEL {
                noneps_in[arc.nextstate] = true;
            }
        }
    }

    // States sorted in topological order when (acyclic) or generic topological
    // order (cyclic).
    let mut states = vec![];

    let fst_props = fst.properties()?;

    if fst_props.contains(FstProperties::TOP_SORTED) {
        states = (0..fst.num_states()).collect();
    } else if fst_props.contains(FstProperties::TOP_SORTED) {
        let mut visitor = TopOrderVisitor::new();
        dfs_visit(fst, &mut visitor, EpsilonArcFilter{}, false);

        for i in 0..visitor.order.len() {
            states[visitor.order[i]] = i;
        }
    } else {
        let mut visitor = SccVisitor::new(fst, true, false);
        dfs_visit(fst, &mut visitor, EpsilonArcFilter {}, false);

        let scc = visitor.scc.as_ref().unwrap();

        let mut first = vec![None; scc.len()];
        let mut next = vec![None; scc.len()];

        for i in 0..scc.len() {
            if first[scc[i] as usize].is_some() {
                next[i] = first[scc[i] as usize];
            }
            first[scc[i] as usize] = Some(i);
        }

        for i in 0..first.len() {
            let mut opt_j = first[i];
            while let Some(j) = opt_j {
                states.push(j);
                opt_j = next[j];
            }
        }
    }
    let mut rmeps_state = RmEpsilonState {
        fst,
        visited: vec![],
        visited_states: vec![],
        element_map: HashMap::new(),
        expand_id: 0
    };
    for state in states.into_iter().rev() {
        if !noneps_in[state] {
            continue;
        }
        rmeps_state.expand(state)?;
    }
    Ok(())
}

#[derive(Hash, Debug, PartialOrd, PartialEq, Eq)]
struct Element {
    ilabel: Label,
    olabel: Label,
    nextstate: StateId
}

struct RmEpsilonState<'a, F: Fst> {
    fst: &'a mut F,
    visited: Vec<bool>,
    visited_states: Vec<StateId>,
    element_map: HashMap<Element, (StateId, usize)>,
    expand_id: usize
}

impl<'a, F: MutableFst> RmEpsilonState<'a, F>
where
    <<F as CoreFst>::W as Semiring>::ReverseWeight: 'static,
{
    pub fn expand(&mut self, source: StateId) -> Fallible<()> {
        let zero = F::W::zero();
        let distance = shortest_distance(self.fst, false)?;

        let arc_filter = EpsilonArcFilter {};

        let mut eps_queue = vec![source];

        let mut arcs = vec![];
        let mut final_weight = F::W::zero();
        while let Some(state) = eps_queue.pop() {
            while self.visited.len() <= state {
                self.visited.push(false);
            }
            if self.visited[state] {
                continue;
            }
            self.visited[state] = true;
            self.visited_states.push(state);
            for arc in self.fst.arcs_iter(state)? {
                // TODO: Remove this clone
                let mut arc = arc.clone();
                arc.weight = distance[state].times(&arc.weight)?;
                if arc_filter.keep(&arc) {
                    while self.visited.len() <= state {
                        self.visited.push(false);
                    }
                    if !self.visited[arc.nextstate] {
                        eps_queue.push(arc.nextstate);
                    }
                } else {
                    let elt = Element {ilabel: arc.ilabel, olabel: arc.olabel, nextstate: arc.nextstate};
                    let val = (self.expand_id, arcs.len());

                    match self.element_map.entry(elt) {
                        Entry::Vacant(e) => {
                            e.insert(val);
                            arcs.push(arc);
                        },
                        Entry::Occupied(mut e) => {
                            if e.get().0 == self.expand_id {
                                unsafe {
                                    arcs.get_unchecked_mut(e.get().1).weight.plus_assign(&arc.weight)?;
                                }
                            } else {
                                e.get_mut().0 = self.expand_id;
                                e.get_mut().1 = arcs.len();
                                arcs.push(arc);
                            }
                        }
                    };
                }
            }
            final_weight.plus_assign(distance[state].times(self.fst.final_weight(state)?.unwrap_or(&zero))?)?;
        }

        while let Some(s) = self.visited_states.pop() {
            self.visited[s] = false;
        }

        self.expand_id += 1;

        unsafe {
            // TODO: Use these arcs instead of cloning
            self.fst.pop_arcs_unchecked(source);
            self.fst.set_arcs_unchecked(source, arcs.into_iter().rev().collect());
            if final_weight != zero {
                self.fst.set_final_unchecked(source, final_weight);
            } else {
                self.fst.delete_final_weight_unchecked(source);
            }
        }
        Ok(())
    }
}
