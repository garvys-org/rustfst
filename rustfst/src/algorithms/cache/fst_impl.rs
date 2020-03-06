use std::collections::{HashSet, VecDeque};
use std::fmt::Debug;
use std::slice::Iter as IterSlice;

use failure::Fallible;

use crate::{Arc, StateId};
use crate::algorithms::cache::CacheImpl;
use crate::fst_traits::{ExpandedFst, Fst, MutableFst};
use crate::semirings::Semiring;

pub trait FstImpl : Debug + PartialEq {
    type W: Semiring + 'static;
    fn cache_impl_mut(&mut self) -> &mut CacheImpl<Self::W>;
    fn cache_impl_ref(&self) -> &CacheImpl<Self::W>;
    fn expand(&mut self, state: StateId) -> Fallible<()>;
    fn compute_start(&mut self) -> Fallible<Option<StateId>>;
    fn compute_final(&mut self, state: StateId) -> Fallible<Option<Self::W>>;

    fn num_known_states(&self) -> usize {
        self.cache_impl_ref().num_known_states()
    }

    fn start(&mut self) -> Fallible<Option<StateId>> {
        if !self.cache_impl_ref().has_start() {
            let start = self.compute_start()?;
            self.cache_impl_mut().set_start(start);
        }
        Ok(self.cache_impl_ref().start().unwrap())
    }

    fn final_weight(&mut self, state: StateId) -> Fallible<Option<&Self::W>> {
        if !self.cache_impl_ref().has_final(state) {
            let final_weight = self.compute_final(state)?;
            self.cache_impl_mut()
                .set_final_weight(state, final_weight)?;
        }
        self.cache_impl_ref().final_weight(state)
    }

    fn arcs_iter(&mut self, state: StateId) -> Fallible<IterSlice<Arc<Self::W>>> {
        self.expand_if_necessary(state)?;
        self.cache_impl_ref().arcs_iter(state)
    }

    fn expand_if_necessary(&mut self, state: StateId) -> Fallible<()> {
        if !self.cache_impl_ref().expanded(state) {
            self.expand(state)?;
            self.cache_impl_mut().mark_expanded(state);
        }
        Ok(())
    }

    fn num_arcs(&mut self, state: StateId) -> Fallible<usize> {
        self.expand_if_necessary(state)?;
        self.cache_impl_ref().num_arcs(state)
    }

    /// Turns the Dynamic FST into a static one.
    fn compute<F2: MutableFst<W = Self::W> + ExpandedFst<W = Self::W>>(&mut self) -> Fallible<F2>
    where
        Self::W: Semiring,
    {
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
