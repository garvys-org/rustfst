use std::slice::Iter as IterSlice;

use crate::algorithms::cache::cache_stores::CacheStore;
use crate::algorithms::cache::CacheState;
use crate::semirings::Semiring;
use crate::Arc;
use crate::StateId;

#[derive(Clone, Debug, PartialOrd, PartialEq, Eq)]
pub struct VectorCacheStore<W> {
    cache_states: Vec<CacheState<W>>,
}

impl<W> CacheStore<W> for VectorCacheStore<W> {
    fn new() -> Self {
        Self {
            cache_states: Vec::new(),
        }
    }

    fn len(&self) -> usize {
        self.cache_states.len()
    }

    fn resize(&mut self, new_len: usize) {
        self.cache_states.resize_with(new_len, CacheState::new);
    }

    fn final_weight_unchecked(&self, state: StateId) -> Option<&W> {
        self.get_cache_state_unchecked(state).final_weight()
    }

    fn set_final_weight_unchecked(&mut self, state: StateId, final_weight: Option<W>) {
        self.get_cache_state_unchecked_mut(state)
            .set_final_weight(final_weight);
    }

    fn has_final(&self, state: StateId) -> bool {
        if state >= self.cache_states.len() {
            return false;
        }
        self.get_cache_state_unchecked(state).has_final()
    }

    fn expanded(&self, state: StateId) -> bool {
        if state >= self.cache_states.len() {
            return false;
        }
        self.get_cache_state_unchecked(state).expanded()
    }

    fn mark_expanded_unchecked(&mut self, state: StateId) {
        self.get_cache_state_unchecked_mut(state).mark_expanded()
    }

    fn push_arc(&mut self, state: StateId, arc: Arc<W>) {
        self.get_cache_state_unchecked_mut(state).push_arc(arc)
    }

    fn num_arcs(&self, state: StateId) -> usize {
        self.get_cache_state_unchecked(state).num_arcs()
    }

    fn arcs_iter_unchecked(&self, state: StateId) -> IterSlice<Arc<W>> {
        self.get_cache_state_unchecked(state).arcs_iter()
    }
}

impl<W> VectorCacheStore<W> {
    pub fn get_cache_state_unchecked(&self, state: StateId) -> &CacheState<W> {
        unsafe { self.cache_states.get_unchecked(state) }
    }

    pub fn get_cache_state_unchecked_mut(&mut self, state: StateId) -> &mut CacheState<W> {
        unsafe { self.cache_states.get_unchecked_mut(state) }
    }

    pub fn reserve_arcs_unchecked(&mut self, state: StateId, n: usize) {
        self.get_cache_state_unchecked_mut(state).reserve_arcs(n)
    }
}
