use std::collections::{HashSet, VecDeque};
use std::slice::Iter as IterSlice;

use failure::Fallible;

use crate::{Arc, StateId};
use crate::algorithms::cache::CacheImpl;
use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::semirings::Semiring;

pub trait FstImpl<W: Semiring + 'static> {
    fn cache_impl(&mut self) -> &mut CacheImpl<W>;
    fn expand(&mut self, state: StateId) -> Fallible<()>;
    fn compute_start(&mut self) -> Fallible<Option<StateId>>;
    fn compute_final(&mut self, state: StateId) -> Fallible<Option<W>>;

    fn start(&mut self) -> Fallible<Option<StateId>> {
        if !self.cache_impl().has_start() {
            let start = self.compute_start()?;
            self.cache_impl().set_start(start);
        }
        Ok(self.cache_impl().start().unwrap())
    }

    fn final_weight(&mut self, state: StateId) -> Fallible<Option<&W>> {
        if !self.cache_impl().has_final(state) {
            let final_weight = self.compute_final(state)?;
            self.cache_impl().set_final_weight(state, final_weight)?;
        }
        self.cache_impl().final_weight(state)
    }

    fn arcs_iter(&mut self, state: StateId) -> Fallible<IterSlice<Arc<W>>> {
        if !self.cache_impl().expanded(state) {
            self.expand(state)?;
            self.cache_impl().mark_expanded(state);
        }
        self.cache_impl().arcs_iter(state)
    }

    /// Turns the Dynamic FST into a static one.
    fn compute<F2: MutableFst<W = W> + ExpandedFst<W = W>>(&mut self) -> Fallible<F2> {
        let start_state = self.start()?;
        let mut fst_out = F2::new();
        if start_state.is_none() {
            return Ok(fst_out);
        }
        let start_state = start_state.unwrap();
        for _ in 0..=start_state {
            fst_out.add_state();
        }
        fst_out.set_start(start_state)?;
        let mut queue = VecDeque::new();
        let mut visited_states = HashSet::new();
        visited_states.insert(start_state);
        queue.push_back(start_state);
        while !queue.is_empty() {
            let s = queue.pop_front().unwrap();
            for arc in self.arcs_iter(s)? {
                if !visited_states.contains(&arc.nextstate) {
                    queue.push_back(arc.nextstate);
                    visited_states.insert(arc.nextstate);
                }
                let n = fst_out.num_states();
                for _ in n..=arc.nextstate {
                    fst_out.add_state();
                }
                fst_out.add_arc(s, arc.clone())?;
            }
            if let Some(f_w) = self.final_weight(s)? {
                fst_out.set_final(s, f_w.clone())?;
            }
        }
        Ok(fst_out)
    }
}
