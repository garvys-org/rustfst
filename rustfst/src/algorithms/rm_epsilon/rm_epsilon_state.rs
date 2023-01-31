use std::borrow::Borrow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use anyhow::Result;

use crate::algorithms::rm_epsilon::{Element, RmEpsilonInternalConfig};
use crate::algorithms::shortest_distance::ShortestDistanceState;
use crate::algorithms::tr_filters::{EpsilonTrFilter, TrFilter};
use crate::algorithms::Queue;
use crate::prelude::Fst;
use crate::semirings::Semiring;
use crate::{StateId, Tr, Trs};
use std::marker::PhantomData;

#[derive(Clone)]
pub(crate) struct RmEpsilonState<W: Semiring, Q: Queue, F: Fst<W>, B: Borrow<F>> {
    pub visited: Vec<bool>,
    pub visited_states: Vec<StateId>,
    pub element_map: HashMap<Element, (StateId, usize)>,
    pub expand_id: StateId,
    pub sd_state: ShortestDistanceState<W, Q, EpsilonTrFilter, F, B>,
    ghost: PhantomData<F>,
}

impl<W: Semiring, Q: Queue, F: Fst<W>, B: Borrow<F>> std::fmt::Debug
    for RmEpsilonState<W, Q, F, B>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RmEpsilonState {{ visited : {:?}, visited_states : {:?}, element_map : {:?}, expand_id : {:?}, sd_state : {:?} }}",
               self.visited, self.visited_states, self.element_map, self.expand_id, self.sd_state)
    }
}

impl<W: Semiring, Q: Queue, F: Fst<W>, B: Borrow<F>> RmEpsilonState<W, Q, F, B> {
    pub fn new(fst: B, opts: RmEpsilonInternalConfig<W, Q>) -> Self {
        Self {
            sd_state: ShortestDistanceState::new_from_config(opts.sd_opts, true, fst),
            visited: vec![],
            visited_states: vec![],
            element_map: HashMap::new(),
            expand_id: 0,
            ghost: PhantomData,
        }
    }

    pub fn fst(&self) -> &F {
        self.sd_state.fst()
    }

    pub fn expand(&mut self, source: StateId) -> Result<(Vec<Tr<W>>, W)> {
        let distance = self.sd_state.shortest_distance(Some(source))?;

        let tr_filter = EpsilonTrFilter {};

        let mut eps_queue = vec![source];

        let mut trs = vec![];
        let mut final_weight = W::zero();
        while let Some(state) = eps_queue.pop() {
            while self.visited.len() <= (state as usize) {
                self.visited.push(false);
            }
            if self.visited[state as usize] {
                continue;
            }
            self.visited[state as usize] = true;
            self.visited_states.push(state);
            for tr in self.sd_state.fst().get_trs(state)?.trs() {
                // TODO: Remove this clone
                let mut tr = tr.clone();
                tr.weight = distance[state as usize].times(&tr.weight)?;
                if tr_filter.keep(&tr) {
                    while self.visited.len() <= (tr.nextstate as usize) {
                        self.visited.push(false);
                    }
                    if !self.visited[(tr.nextstate as usize)] {
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
                distance[state as usize].times(
                    self.sd_state
                        .fst()
                        .final_weight(state)?
                        .unwrap_or_else(W::zero),
                )?,
            )?;
        }

        while let Some(s) = self.visited_states.pop() {
            self.visited[s as usize] = false;
        }

        self.expand_id += 1;

        Ok((trs, final_weight))
    }
}
