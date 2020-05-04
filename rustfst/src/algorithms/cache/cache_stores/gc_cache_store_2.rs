use std::marker::PhantomData;
use std::slice::Iter as IterSlice;

use itertools::Itertools;

use crate::algorithms::cache::cache_stores::vector_cache_store_2::VectorCacheStore2;
use crate::algorithms::cache::cache_stores::{CacheOptions, CacheStore2};
use crate::algorithms::cache::{CacheFlags, CacheState};
use crate::{Arc, StateId};

pub struct GCCacheStore2<W, S> {
    store: S,
    cache_limit: usize,
    cache_gc: bool,
    cache_size: usize,
    cache_gc_request: bool,
    w: PhantomData<W>,
}

static CACHE_FRACTION: f32 = 0.666;

impl<W, S: CacheStore2<W>> GCCacheStore2<W, S> {
    pub fn new(opts: &CacheOptions) -> Self {
        Self {
            store: S::new(opts),
            cache_gc_request: opts.gc,
            cache_limit: opts.gc_limit,
            cache_gc: false,
            cache_size: 0,
            w: PhantomData,
        }
    }

    pub fn get_state(&mut self, s: StateId) -> *mut CacheState<W> {
        self.store.get_state(s)
    }

    pub fn get_mutable_state(&mut self, s: StateId) -> *mut CacheState<W> {
        let state = self.store.get_mutable_state(s);
        let state = unsafe { &mut *state };

        if self.cache_gc_request && !state.flags().contains(CacheFlags::CACHE_INIT) {
            state.set_flags(CacheFlags::CACHE_INIT, CacheFlags::CACHE_INIT);
            self.cache_size += std::mem::size_of::<CacheState<W>>()
                + state.num_arcs() * std::mem::size_of::<Arc<W>>();
            // GC is enabled once an uninited state (from underlying store) is seen.
            self.cache_gc = true;
            if self.cache_size > self.cache_limit {
                self.gc(state, false, CACHE_FRACTION);
            }
        }
        state as *mut CacheState<W>
    }

    pub fn add_arc(&mut self, state: *mut CacheState<W>, arc: Arc<W>) {
        self.store.add_arc(state, arc);
        let state = unsafe { &mut *state };
        if self.cache_gc && state.flags().contains(CacheFlags::CACHE_INIT) {
            self.cache_size += std::mem::size_of::<Arc<W>>();
            if self.cache_size > self.cache_limit {
                self.gc(state, false, CACHE_FRACTION);
            }
        }
    }

    pub fn mark_expanded(&mut self, state: *mut CacheState<W>) {
        self.store.mark_expanded(state);
        let state = unsafe { &mut *state };
        if self.cache_gc && state.flags().contains(CacheFlags::CACHE_INIT) {
            self.cache_size += state.num_arcs() * std::mem::size_of::<Arc<W>>();
            if self.cache_size > self.cache_limit {
                self.gc(state, false, CACHE_FRACTION);
            }
        }
    }

    pub fn delete_arcs(&mut self, state: *mut CacheState<W>) {
        let state = unsafe { &mut *state };
        if self.cache_gc && state.flags().contains(CacheFlags::CACHE_INIT) {
            self.cache_size -= state.num_arcs() * std::mem::size_of::<Arc<W>>();
        }
        self.store.delete_arcs(state);
    }

    pub fn clear(&mut self) {
        self.store.clear();
        self.cache_size = 0;
    }

    pub fn count_states(&self) -> usize {
        self.store.count_states()
    }

    pub fn iter(&self) -> IterSlice<StateId> {
        self.store.iter()
    }

    // Removes from the cache store (not referenced-counted and not the current)
    // states that have not been accessed since the last GC until at most
    // cache_fraction * cache_limit_ bytes are cached. If that fails to free
    // enough, attempts to uncaching recently visited states as well. If still
    // unable to free enough memory, then widens cache_limit_.
    fn gc(&mut self, current: *mut CacheState<W>, free_recent: bool, cache_fraction: f32) {
        if !self.cache_gc {
            return;
        }
        let mut cache_target = ((self.cache_limit as f32) * cache_fraction).round() as usize;

        // TODO: Remove this collect
        let s_collected = self.store.iter().cloned().collect_vec();
        for (idx, s) in s_collected.iter().enumerate() {
            let state_ptr = self.store.get_mutable_state(*s);
            let state = unsafe { &mut *state_ptr };
            if self.cache_size > cache_target
                && state.ref_count() == 0
                && (free_recent || !state.flags().contains(CacheFlags::CACHE_RECENT))
                && state_ptr != current
            {
                if state.flags().contains(CacheFlags::CACHE_INIT) {
                    let size = std::mem::size_of::<CacheState<W>>()
                        + state.num_arcs() * std::mem::size_of::<Arc<W>>();
                    if size < self.cache_size {
                        self.cache_size -= size;
                    }
                }
                self.store.delete(idx, *s);
            } else {
                state.set_flags(CacheFlags::empty(), CacheFlags::CACHE_RECENT);
            }
        }

        if !free_recent && self.cache_size > cache_target {
            self.gc(current, true, cache_fraction);
        } else if cache_target > 0 {
            while self.cache_size > cache_target {
                self.cache_limit *= 2;
                cache_target *= 2;
            }
        } else if self.cache_size > 0 {
            panic!("GCCacheStore:GC: Unable to free all cached states")
        }
    }
}
