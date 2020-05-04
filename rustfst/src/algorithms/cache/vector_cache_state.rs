use std::slice::Iter as IterSlice;

use crate::algorithms::cache::CacheState;
use crate::semirings::Semiring;
use crate::StateId;
use crate::Tr;

#[derive(Clone, Debug, PartialOrd, PartialEq, Eq)]
pub struct VectorCacheState<W> {
    cache_states: Vec<CacheState<W>>,
}

impl<W> VectorCacheState<W> {
    pub fn new() -> Self {
        Self {
            cache_states: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.cache_states.len()
    }

    pub fn resize(&mut self, new_len: usize) {
        self.cache_states.resize_with(new_len, CacheState::new);
    }

    pub fn resize_if_necessary(&mut self, new_len: usize) {
        if self.cache_states.len() < new_len {
            self.resize(new_len)
        }
    }

    pub fn get_cache_state_unchecked(&self, state: StateId) -> &CacheState<W> {
        unsafe { self.cache_states.get_unchecked(state) }
    }

    pub fn get_cache_state_unchecked_mut(&mut self, state: StateId) -> &mut CacheState<W> {
        unsafe { self.cache_states.get_unchecked_mut(state) }
    }

    pub fn set_final_weight_unchecked(&mut self, state: StateId, final_weight: Option<W>) {
        self.get_cache_state_unchecked_mut(state)
            .set_final_weight(final_weight);
    }

    pub fn push_tr(&mut self, state: StateId, tr: Tr<W>) {
        self.get_cache_state_unchecked_mut(state).push_tr(tr)
    }

    pub fn tr_iter_unchecked(&self, state: StateId) -> IterSlice<Tr<W>> {
        self.get_cache_state_unchecked(state).tr_iter()
    }

    pub fn mark_expanded_unchecked(&mut self, state: StateId) {
        self.get_cache_state_unchecked_mut(state).mark_expanded()
    }

    pub fn reserve_trs_unchecked(&mut self, state: StateId, n: usize) {
        self.get_cache_state_unchecked_mut(state).reserve_trs(n)
    }

    pub fn expanded(&self, state: StateId) -> bool {
        if state >= self.cache_states.len() {
            return false;
        }
        self.get_cache_state_unchecked(state).expanded()
    }

    pub fn has_final(&self, state: StateId) -> bool {
        if state >= self.cache_states.len() {
            return false;
        }
        self.get_cache_state_unchecked(state).has_final()
    }

    pub fn final_weight_unchecked(&self, state: StateId) -> Option<&W> {
        self.get_cache_state_unchecked(state).final_weight()
    }

    pub fn num_trs(&self, state: StateId) -> usize {
        self.get_cache_state_unchecked(state).num_trs()
    }
}
