use std::marker::PhantomData;
use std::slice::Iter as IterSlice;

use failure::Fallible;

use crate::algorithms::cache::cache_stores::{CacheStore, DefaultCacheStore, VectorCacheStore};
use crate::Arc;
use crate::StateId;

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq)]
pub struct CacheImpl<W, S = VectorCacheStore<W>> {
    has_start: bool,
    cache_start_state: Option<StateId>,
    cache_store: S,
    ghost: PhantomData<W>,
}

impl<W, S: CacheStore<W>> CacheImpl<W, S> {
    pub fn new() -> Self {
        Self {
            has_start: false,
            cache_start_state: None,
            cache_store: S::new(),
            ghost: PhantomData,
        }
    }

    pub fn num_known_states(&self) -> usize {
        self.cache_store.len()
    }

    pub fn set_start(&mut self, start_state: Option<StateId>) {
        self.cache_start_state = start_state;
        self.has_start = true;
        if let Some(s) = start_state {
            self.cache_store.resize_if_necessary(s + 1);
        }
    }

    pub fn start(&self) -> Fallible<Option<StateId>> {
        if !self.has_start {
            bail!("Can't call start() before set_start()");
        }
        Ok(self.cache_start_state)
    }

    pub fn set_final_weight(&mut self, state: StateId, final_weight: Option<W>) -> Fallible<()> {
        self.cache_store.resize_if_necessary(state + 1);
        self.cache_store
            .set_final_weight_unchecked(state, final_weight);
        Ok(())
    }

    pub fn final_weight(&self, state: StateId) -> Fallible<Option<&W>> {
        if !self.cache_store.has_final(state) {
            bail!("Can't call final_weight() before set_final_weight()")
        }
        Ok(self.cache_store.final_weight_unchecked(state))
    }

    pub fn push_arc(&mut self, state: StateId, arc: Arc<W>) -> Fallible<()> {
        if self.cache_store.expanded(state) {
            bail!("Can't add arcs to a fully expanded state")
        }
        self.cache_store.resize_if_necessary(state + 1);
        self.cache_store.resize_if_necessary(arc.nextstate + 1);
        self.cache_store.push_arc(state, arc);
        Ok(())
    }

    pub fn num_arcs(&self, state: StateId) -> Fallible<usize> {
        if !self.cache_store.expanded(state) {
            bail!("Can't call num_arcs on a state that is not fully expanded");
        }
        Ok(self.cache_store.num_arcs(state))
    }

    pub fn expanded(&self, state: StateId) -> bool {
        self.cache_store.expanded(state)
    }

    pub fn has_final(&self, state: StateId) -> bool {
        self.cache_store.has_final(state)
    }

    pub fn mark_expanded(&mut self, state: StateId) {
        self.cache_store.resize_if_necessary(state + 1);
        self.cache_store.mark_expanded_unchecked(state)
    }

    pub fn arcs_iter(&self, state: StateId) -> Fallible<IterSlice<Arc<W>>> {
        if !self.cache_store.expanded(state) {
            bail!("Can't iterate arcs on a not fully expanded state")
        }
        Ok(self.cache_store.arcs_iter_unchecked(state))
    }

    pub fn has_start(&self) -> bool {
        self.has_start
    }
}

#[cfg(test)]
mod tests {
    use crate::semirings::Semiring;
    use crate::semirings::TropicalWeight;

    use super::*;

    fn test_cache_impl_start() -> Fallible<()> {
        let mut cache_impl = CacheImpl::<TropicalWeight>::new();
        assert!(!cache_impl.has_start());
        assert_eq!(cache_impl.num_known_states(), 0);
        cache_impl.set_start(Some(1));
        assert_eq!(cache_impl.start()?, Some(1));
        assert!(cache_impl.has_start());
        assert_eq!(cache_impl.num_known_states(), 1);
        Ok(())
    }

    fn test_cache_expanded() -> Fallible<()> {
        let mut cache_impl = CacheImpl::<TropicalWeight>::new();
        cache_impl.set_start(Some(1));
        assert!(!cache_impl.expanded(2));
        cache_impl.mark_expanded(2);
        assert!(cache_impl.expanded(2));

        cache_impl.push_arc(1, Arc::new(2, 3, TropicalWeight::new(2.3), 3));

        assert!(!cache_impl.expanded(3));
        cache_impl.mark_expanded(3);
        assert!(cache_impl.expanded(3));

        Ok(())
    }
}
