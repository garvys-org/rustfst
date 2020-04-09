use crate::algorithms::cache::cache_stores::CacheOptions;
use crate::algorithms::cache::{CacheFlags, CacheState};
use crate::{Arc, StateId};
use itertools::Itertools;
use std::collections::LinkedList;
use std::slice::Iter as IterSlice;

pub struct VectorCacheStore2<W> {
    state_list: Vec<usize>,
    state_vec: Vec<Option<CacheState<W>>>,
    cache_gc: bool,
}

impl<W> VectorCacheStore2<W> {
    pub fn new(opts: &CacheOptions) -> Self {
        Self {
            state_list: Vec::new(),
            state_vec: Vec::new(),
            cache_gc: opts.gc,
        }
    }

    pub fn in_bounds(&self, s: StateId) -> bool {
        s < self.state_vec.len()
    }

    // Return None if state is not stored
    pub fn get_state(&self, s: StateId) -> *const CacheState<W> {
        if self.in_bounds(s) {
            let a = &self.state_vec[s];
            match a {
                Some(e) => e as *const CacheState<W>,
                None => std::ptr::null(),
            }
        } else {
            std::ptr::null()
        }
    }

    // Creates state if state is not stored
    pub fn get_mutable_state(&mut self, s: StateId) -> *mut CacheState<W> {
        let mut state = None;
        if self.in_bounds(s) {
            state = self.state_vec[s].as_mut();
        } else {
            self.state_vec.resize_with(s + 1, || None);
        }

        if let Some(_state) = state {
            _state as *mut CacheState<W>
        } else {
            self.state_vec[s] = Some(CacheState::new());
            if self.cache_gc {
                self.state_list.push(s);
            }
            self.state_vec[s].as_mut().unwrap() as *mut CacheState<W>
        }
    }

    pub fn add_arc(&mut self, state: *mut CacheState<W>, arc: Arc<W>) {
        let state = unsafe { &mut *state };
        state.push_arc(arc);
    }

    // equivalent of set_arcs
    pub fn mark_expanded(&mut self, state: *mut CacheState<W>) {
        let state = unsafe { &mut *state };
        state.mark_expanded()
    }

    pub fn delete_arcs(&mut self, state: *mut CacheState<W>) {
        let state = unsafe { &mut *state };
        state.delete_arcs();
    }

    pub fn clear(&mut self) {
        self.state_list.clear();
        self.state_vec.clear();
    }

    pub fn count_states(&self) -> usize {
        self.state_vec.iter().filter(|s| s.is_some()).count()
    }

    pub fn iter(&self) -> IterSlice<StateId> {
        self.state_list.iter()
    }

    pub fn delete(&mut self, idx: usize, s: StateId) {
        self.state_vec[s] = None;
        self.state_list.remove(idx);
    }
}
