use std::slice::Iter as IterSlice;

use crate::algorithms::cache::cache_stores::CacheStore;
use crate::algorithms::cache::CacheState;
use crate::Arc;

#[derive(Clone, Debug, PartialOrd, PartialEq, Eq)]
pub struct GcCacheStore<S> {
    cache_store: S,
    cache_limit: usize,
    cache_size: usize,
}

impl<S> GcCacheStore<S> {
    pub fn cache_limit(&self) -> usize {
        self.cache_limit
    }

    pub fn cache_size(&self) -> usize {
        self.cache_size
    }
}

impl<W, S: CacheStore<W>> CacheStore<W> for GcCacheStore<S> {
    fn new() -> Self {
        unimplemented!()
    }

    fn len(&self) -> usize {
        self.cache_store.len()
    }

    fn resize(&mut self, new_len: usize) {
        self.cache_size += (new_len - self.len()) * std::mem::size_of::<CacheState<W>>();
        self.cache_store.resize(new_len)
    }

    fn final_weight_unchecked(&self, state: usize) -> Option<&W> {
        self.cache_store.final_weight_unchecked(state)
    }

    fn set_final_weight_unchecked(&mut self, state: usize, final_weight: Option<W>) {
        self.cache_store
            .set_final_weight_unchecked(state, final_weight)
    }

    fn has_final(&self, state: usize) -> bool {
        self.cache_store.has_final(state)
    }

    fn expanded(&self, state: usize) -> bool {
        self.cache_store.expanded(state)
    }

    fn mark_expanded_unchecked(&mut self, state: usize) {
        self.cache_store.mark_expanded_unchecked(state)
    }

    fn push_arc(&mut self, state: usize, arc: Arc<W>) {
        self.cache_size += std::mem::size_of::<Arc<W>>();
        self.cache_store.push_arc(state, arc)
    }

    fn num_arcs(&self, state: usize) -> usize {
        self.cache_store.num_arcs(state)
    }

    fn arcs_iter_unchecked(&self, state: usize) -> IterSlice<Arc<W>> {
        self.cache_store.arcs_iter_unchecked(state)
    }
}
