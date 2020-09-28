use std::collections::HashMap;
use std::sync::Mutex;

use crate::algorithms::lazy::cache::cache_internal_types::{CachedData, StartState};
use crate::algorithms::lazy::{CacheStatus, FstCache};
use crate::semirings::Semiring;
use crate::{StateId, Trs, TrsVec, EPS_LABEL};
use unsafe_unwrap::UnsafeUnwrap;

#[derive(Debug)]
pub struct SimpleHashMapCache<W: Semiring> {
    // First option : has start been computed
    // Second option: value of the start state (possibly none)
    // The second element of each tuple is the number of known states.
    start: Mutex<CachedData<CacheStatus<StartState>>>,
    trs: Mutex<CachedData<HashMap<StateId, CacheTrs<W>>>>,
    final_weights: Mutex<CachedData<HashMap<StateId, Option<W>>>>,
}

#[derive(Debug, Clone)]
pub struct CacheTrs<W: Semiring> {
    pub trs: TrsVec<W>,
    pub niepsilons: usize,
    pub noepsilons: usize,
}

impl<W: Semiring> SimpleHashMapCache<W> {
    pub fn clear(&self) {
        let mut data_start = self.start.lock().unwrap();
        data_start.clear();

        let mut data_trs = self.trs.lock().unwrap();
        data_trs.clear();

        let mut data_final_weights = self.final_weights.lock().unwrap();
        data_final_weights.clear();
    }
}

impl<W: Semiring> Clone for SimpleHashMapCache<W> {
    fn clone(&self) -> Self {
        Self {
            start: Mutex::new(self.start.lock().unwrap().clone()),
            trs: Mutex::new(self.trs.lock().unwrap().clone()),
            final_weights: Mutex::new(self.final_weights.lock().unwrap().clone()),
        }
    }
}

impl<W: Semiring> Default for SimpleHashMapCache<W> {
    fn default() -> Self {
        Self {
            start: Mutex::new(CachedData::default()),
            trs: Mutex::new(CachedData::default()),
            final_weights: Mutex::new(CachedData::default()),
        }
    }
}

impl<W: Semiring> FstCache<W> for SimpleHashMapCache<W> {
    fn get_start(&self) -> CacheStatus<Option<StateId>> {
        let res = self.start.lock().unwrap();
        res.data
    }

    fn insert_start(&self, id: Option<StateId>) {
        let mut data = self.start.lock().unwrap();
        if let Some(s) = id {
            data.num_known_states = std::cmp::max(data.num_known_states, s + 1);
        }
        data.data = CacheStatus::Computed(id);
    }

    fn get_trs(&self, id: usize) -> CacheStatus<TrsVec<W>> {
        match self.trs.lock().unwrap().data.get(&id) {
            Some(e) => CacheStatus::Computed(e.trs.shallow_clone()),
            None => CacheStatus::NotComputed,
        }
    }

    fn insert_trs(&self, id: usize, trs: TrsVec<W>) {
        let mut cached_data = self.trs.lock().unwrap();
        let mut niepsilons = 0;
        let mut noepsilons = 0;
        for tr in trs.trs() {
            cached_data.num_known_states =
                std::cmp::max(cached_data.num_known_states, tr.nextstate + 1);
            if tr.ilabel == EPS_LABEL {
                niepsilons += 1;
            }
            if tr.olabel == EPS_LABEL {
                noepsilons += 1;
            }
        }
        cached_data.data.insert(
            id,
            CacheTrs {
                trs,
                niepsilons,
                noepsilons,
            },
        );
    }
    fn get_final_weight(&self, id: usize) -> CacheStatus<Option<W>> {
        match self.final_weights.lock().unwrap().data.get(&id) {
            Some(e) => CacheStatus::Computed(e.clone()),
            None => CacheStatus::NotComputed,
        }
    }

    fn insert_final_weight(&self, id: StateId, weight: Option<W>) {
        let mut cached_data = self.final_weights.lock().unwrap();
        cached_data.num_known_states = std::cmp::max(cached_data.num_known_states, id + 1);
        cached_data.data.insert(id, weight);
    }

    fn num_known_states(&self) -> usize {
        let mut n = 0;
        n = std::cmp::max(n, self.start.lock().unwrap().num_known_states);
        n = std::cmp::max(n, self.trs.lock().unwrap().num_known_states);
        n = std::cmp::max(n, self.final_weights.lock().unwrap().num_known_states);
        n
    }

    fn num_trs(&self, id: usize) -> Option<usize> {
        let cached_data = self.trs.lock().unwrap();
        cached_data.data.get(&id).map(|v| v.trs.len())
    }

    fn num_input_epsilons(&self, id: usize) -> CacheStatus<usize> {
        let cached_data = self.trs.lock().unwrap();
        match cached_data.data.get(&id) {
            Some(e) => CacheStatus::Computed(e.niepsilons),
            None => CacheStatus::NotComputed,
        }
    }

    unsafe fn num_input_epsilons_unchecked(&self, id: usize) -> usize {
        let cached_data = self.trs.lock().unwrap();
        cached_data.data.get(&id).unsafe_unwrap().niepsilons
    }

    fn num_output_epsilons(&self, id: usize) -> CacheStatus<usize> {
        let cached_data = self.trs.lock().unwrap();
        match cached_data.data.get(&id) {
            Some(e) => CacheStatus::Computed(e.noepsilons),
            None => CacheStatus::NotComputed,
        }
    }

    unsafe fn num_output_epsilons_unchecked(&self, id: usize) -> usize {
        let cached_data = self.trs.lock().unwrap();
        cached_data.data.get(&id).unsafe_unwrap().noepsilons
    }

    fn len_trs(&self) -> usize {
        let cached_data = self.trs.lock().unwrap();
        cached_data.data.len()
    }

    fn len_final_weights(&self) -> usize {
        let cached_data = self.final_weights.lock().unwrap();
        cached_data.data.len()
    }

    fn is_final(&self, state_id: usize) -> CacheStatus<bool> {
        match self.final_weights.lock().unwrap().data.get(&state_id) {
            Some(e) => CacheStatus::Computed(e.is_some()),
            None => CacheStatus::NotComputed,
        }
    }

    unsafe fn is_final_unchecked(&self, state_id: usize) -> bool {
        self.final_weights
            .lock()
            .unwrap()
            .data
            .get(&state_id)
            .unwrap()
            .is_some()
    }
}
