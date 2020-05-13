use std::borrow::Borrow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::marker::PhantomData;

use anyhow::Result;

use crate::algorithms::rm_epsilon::{Element, RmEpsilonConfig};
use crate::algorithms::shortest_distance::ShortestDistanceState;
use crate::algorithms::tr_filters::{EpsilonTrFilter, TrFilter};
use crate::algorithms::Queue;
use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::{StateId, Tr, Trs};

#[derive(Clone, Eq)]
pub struct RmEpsilonState<W: Semiring, F: MutableFst<W>, B: Borrow<F>, Q: Queue> {
    pub visited: Vec<bool>,
    pub visited_states: Vec<StateId>,
    pub element_map: HashMap<Element, (StateId, usize)>,
    pub expand_id: usize,
    pub sd_state: ShortestDistanceState<W, Q, F, B, EpsilonTrFilter>,
    pub f: PhantomData<F>,
}

impl<W: Semiring, F: MutableFst<W>, B: Borrow<F>, Q: Queue> std::fmt::Debug
    for RmEpsilonState<W, F, B, Q>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RmEpsilonState {{ visited : {:?}, visited_states : {:?}, element_map : {:?}, expand_id : {:?}, sd_state : {:?} }}",
               self.visited, self.visited_states, self.element_map, self.expand_id, self.sd_state)
    }
}

impl<W: Semiring, F: MutableFst<W>, B: Borrow<F>, Q: Queue + PartialEq> PartialEq
    for RmEpsilonState<W, F, B, Q>
{
    fn eq(&self, other: &Self) -> bool {
        self.visited.eq(&other.visited)
            && self.visited_states.eq(&other.visited_states)
            && self.element_map.eq(&other.element_map)
            && self.expand_id.eq(&other.expand_id)
            && self.sd_state.eq(&other.sd_state)
    }
}

impl<W: Semiring, F: MutableFst<W>, B: Borrow<F>, Q: Queue> RmEpsilonState<W, F, B, Q> {
    pub fn new(fst: B, opts: RmEpsilonConfig<W, Q>) -> Self {
        Self {
            sd_state: ShortestDistanceState::new_from_config(fst, opts.sd_opts, true),
            visited: vec![],
            visited_states: vec![],
            element_map: HashMap::new(),
            expand_id: 0,
            f: PhantomData,
        }
    }

    pub fn expand(&mut self, source: StateId) -> Result<(Vec<Tr<W>>, W)> {
        let distance = self.sd_state.shortest_distance(Some(source))?;

        let tr_filter = EpsilonTrFilter {};

        let mut eps_queue = vec![source];

        let mut trs = vec![];
        let mut final_weight = W::zero();
        while let Some(state) = eps_queue.pop() {
            while self.visited.len() <= state {
                self.visited.push(false);
            }
            if self.visited[state] {
                continue;
            }
            self.visited[state] = true;
            self.visited_states.push(state);
            for tr in self.sd_state.fst.borrow().get_trs(state)?.trs() {
                // TODO: Remove this clone
                let mut tr = tr.clone();
                tr.weight = distance[state].times(&tr.weight)?;
                if tr_filter.keep(&tr) {
                    while self.visited.len() <= tr.nextstate {
                        self.visited.push(false);
                    }
                    if !self.visited[tr.nextstate] {
                        eps_queue.push(tr.nextstate);
                    }
                } else {
                    let elt = Element {
                        ilabel: tr.ilabel,
                        olabel: tr.olabel,
                        nextstate: tr.nextstate,
                    };
                    let val = (self.expand_id, trs.len());

                    match self.element_map.entry(elt) {
                        Entry::Vacant(e) => {
                            e.insert(val);
                            trs.push(tr);
                        }
                        Entry::Occupied(mut e) => {
                            if e.get().0 == self.expand_id {
                                unsafe {
                                    trs.get_unchecked_mut(e.get().1)
                                        .weight
                                        .plus_assign(&tr.weight)?;
                                }
                            } else {
                                e.get_mut().0 = self.expand_id;
                                e.get_mut().1 = trs.len();
                                trs.push(tr);
                            }
                        }
                    };
                }
            }
            final_weight.plus_assign(
                distance[state].times(
                    self.sd_state
                        .fst
                        .borrow()
                        .final_weight(state)?
                        .unwrap_or_else(W::zero),
                )?,
            )?;
        }

        while let Some(s) = self.visited_states.pop() {
            self.visited[s] = false;
        }

        self.expand_id += 1;

        Ok((trs, final_weight))
    }
}
