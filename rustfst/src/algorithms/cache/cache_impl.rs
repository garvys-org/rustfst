use std::slice::Iter as IterSlice;

use failure::Fallible;

use crate::algorithms::cache::VectorCacheState;
use crate::semirings::Semiring;
use crate::Arc;
use crate::StateId;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct CacheImpl<W: Semiring> {
    has_start: bool,
    cache_start_state: Option<StateId>,
    vector_cache_states: VectorCacheState<W>,
}

impl<W: Semiring> CacheImpl<W> {
    pub fn new() -> Self {
        Self {
            has_start: false,
            cache_start_state: None,
            vector_cache_states: VectorCacheState::new(),
        }
    }

    pub fn set_start(&mut self, start_state: Option<StateId>) {
        self.cache_start_state = start_state;
        self.has_start = true;
        if let Some(s) = start_state {
            self.vector_cache_states.resize_if_necessary(s + 1);
        }
    }

    pub fn start(&self) -> Fallible<Option<StateId>> {
        if !self.has_start {
            bail!("Can't call start() before set_start()");
        }
        Ok(self.cache_start_state)
    }

    pub fn set_final_weight(&mut self, state: StateId, final_weight: Option<W>) -> Fallible<()> {
        self.vector_cache_states.resize_if_necessary(state + 1);
        self.vector_cache_states
            .set_final_weight_unchecked(state, final_weight);
        Ok(())
    }

    pub fn final_weight(&self, state: StateId) -> Fallible<Option<&W>> {
        if !self.vector_cache_states.has_final(state) {
            bail!("Can't call final_weight() before set_final_weight()")
        }
        Ok(self.vector_cache_states.final_weight_unchecked(state))
    }

    pub fn push_arc(&mut self, state: StateId, arc: Arc<W>) -> Fallible<()> {
        if self.vector_cache_states.expanded(state) {
            bail!("Can't add arcs to a fully expanded state")
        }
        self.vector_cache_states.resize_if_necessary(state + 1);
        self.vector_cache_states.push_arc(state, arc);
        Ok(())
    }

    pub fn num_arcs(&self, state: StateId) -> Fallible<usize> {
        if !self.vector_cache_states.expanded(state) {
            bail!("Can't call num_arcs on a state that is not fully expanded");
        }
        Ok(self.vector_cache_states.num_arcs(state))
    }

    pub fn expanded(&self, state: StateId) -> bool {
        self.vector_cache_states.expanded(state)
    }

    pub fn has_final(&self, state: StateId) -> bool {
        self.vector_cache_states.has_final(state)
    }

    pub fn mark_expanded(&mut self, state: StateId) {
        self.vector_cache_states.resize_if_necessary(state + 1);
        self.vector_cache_states.mark_expanded_unchecked(state)
    }

    pub fn arcs_iter(&self, state: StateId) -> Fallible<IterSlice<Arc<W>>> {
        if !self.vector_cache_states.expanded(state) {
            bail!("Can't iterate arcs on a not fully expanded state")
        }
        Ok(self.vector_cache_states.arcs_iter_unchecked(state))
    }

    pub fn has_start(&self) -> bool {
        self.has_start
    }
}
