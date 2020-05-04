use crate::algorithms::cache::cache_stores::vector_cache_store_2::VectorCacheStore2;
use std::marker::PhantomData;
use crate::semirings::Semiring;
use crate::algorithms::cache::cache_stores::CacheStore2;
use crate::algorithms::cache::CacheFlags;
use crate::StateId;
use failure::Fallible;

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq)]
pub struct CacheImpl2<W, S = VectorCacheStore2<W>> {
    cache_store: S,
    ghost: PhantomData<W>,
    has_start: bool,
    nknown_states: usize,
    cache_start_state: Option<StateId>,
}

impl<W: Semiring, S: CacheStore2<W>> CacheImpl2<W, S> {
    pub fn num_known_states(&self) -> usize {
        self.nknown_states
    }

    pub fn start(&self) -> Option<StateId> {

        #[cfg(debug_assertions)]
        {
            if !self.has_start {
                panic!("Can't call start() before set_start()")
            }
        }

        self.cache_start_state
    }

    pub fn set_start(&mut self, state: Option<StateId>) {
        self.cache_start_state = state;
        self.has_start = true;
        if let Some(s) = state {
            if s >= self.nknown_states {
                self.nknown_states = s + 1;
            }
        }
    }

    pub fn has_start(&self) -> bool {
        self.has_start
    }

    pub fn set_final_weight(&mut self, s: StateId, final_weight: Option<W>) {
        let state = self.cache_store.get_mutable_state(s);
        let state = unsafe {&mut *state};
        state.set_final_weight(final_weight);
        let flags = CacheFlags::CACHE_FINAL | CacheFlags::CACHE_RECENT;
        state.set_flags(flags, flags);
    }

    pub fn final_weight(&mut self, state: StateId) -> Option<&W> {
        let state = self.cache_store.get_state(state);
        let state = unsafe {&*state};

        #[cfg(debug_assertions)]
        {
            if !state.has_final() {
                panic!("Can't call final_weight() before set_final_weight()")
            }
        }

        state.final_weight()
    }

    pub fn has_final(&mut self, state: StateId) -> bool {
        let state = self.cache_store.get_state(state);
        if state == std::ptr::null_mut() {
            return false;
        }
        let state = unsafe {&mut *state};
        if state.has_final() {
            state.set_flags(CacheFlags::CACHE_RECENT, CacheFlags::CACHE_RECENT);
            true
        } else {
            false
        }

    }


}
