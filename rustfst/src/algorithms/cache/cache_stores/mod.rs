use std::slice::Iter as IterSlice;

use crate::{Arc, StateId};

pub use self::vector_cache_store::VectorCacheStore;
use crate::algorithms::cache::CacheState;

mod gc_cache_store;
mod gc_cache_store_2;
mod vector_cache_store;
mod vector_cache_store_2;

#[derive(Clone, Debug, PartialOrd, PartialEq, Eq)]
pub struct DefaultCacheStore<W>(VectorCacheStore<W>);

pub trait CacheStore<W> {
    fn new() -> Self;

    fn len(&self) -> usize;

    fn resize(&mut self, new_len: usize);

    fn resize_if_necessary(&mut self, new_len: usize) {
        if self.len() < new_len {
            self.resize(new_len)
        }
    }

    fn final_weight_unchecked(&self, state: StateId) -> Option<&W>;

    fn set_final_weight_unchecked(&mut self, state: StateId, final_weight: Option<W>);

    fn has_final(&self, state: StateId) -> bool;

    fn expanded(&self, state: StateId) -> bool;

    fn mark_expanded_unchecked(&mut self, state: StateId);

    fn push_arc(&mut self, state: StateId, arc: Arc<W>);

    fn num_arcs(&self, state: StateId) -> usize;

    fn arcs_iter_unchecked(&self, state: StateId) -> IterSlice<Arc<W>>;
}

pub trait CacheStore2<W> {
    fn new(opts: &CacheOptions) -> Self;
    fn get_state(&self, s: StateId) -> *const CacheState<W>;
    fn get_mutable_state(&mut self, s: StateId) -> *mut CacheState<W>;

    fn add_arc(&mut self, state: *mut CacheState<W>, arc: Arc<W>);
    fn mark_expanded(&mut self, state: *mut CacheState<W>);
    fn delete_arcs(&mut self, state: *mut CacheState<W>);
    fn clear(&mut self);
    fn count_states(&self) -> usize;
    fn iter(&self) -> IterSlice<StateId>;
    fn delete(&mut self, idx: usize, s: StateId);
}

pub struct CacheOptions {
    /// Enables GC.
    pub gc: bool,
    /// Number of bytes allowed before GC.
    pub gc_limit: usize,
}
