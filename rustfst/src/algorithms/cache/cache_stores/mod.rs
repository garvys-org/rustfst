use std::slice::Iter as IterSlice;

use crate::{Arc, StateId};

pub use self::vector_cache_store::VectorCacheStore;

mod vector_cache_store;

#[derive(Clone, Debug, PartialOrd, PartialEq, Eq)]
pub struct DefaultCacheStore<W>(VectorCacheStore<W>);

pub trait CacheStore<W> {
    fn new() -> Self;

    fn len(&self) -> usize;

    fn resize(&mut self, new_len: usize);

    fn resize_if_necessary(&mut self, new_len: usize);

    fn final_weight_unchecked(&self, state: StateId) -> Option<&W>;

    fn set_final_weight_unchecked(&mut self, state: StateId, final_weight: Option<W>);

    fn has_final(&self, state: StateId) -> bool;

    fn expanded(&self, state: StateId) -> bool;

    fn mark_expanded_unchecked(&mut self, state: StateId);

    fn push_arc(&mut self, state: StateId, arc: Arc<W>);

    fn num_arcs(&self, state: StateId) -> usize;

    fn arcs_iter_unchecked(&self, state: StateId) -> IterSlice<Arc<W>>;
}
