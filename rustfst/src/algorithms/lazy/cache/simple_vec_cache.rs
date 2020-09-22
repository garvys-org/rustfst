use std::sync::Mutex;

use crate::algorithms::lazy::cache::cache_internal_types::{CachedData, FinalWeight, StartState};
use crate::algorithms::lazy::FstCache;
use crate::semirings::Semiring;
use crate::{StateId, Trs, TrsVec, EPS_LABEL};

#[derive(Debug)]
pub struct SimpleVecCache<W: Semiring> {
    // First option : has start been computed
    // Second option: value of the start state (possibly none)
    // The second element of each tuple is the number of known states.
    start: Mutex<CachedData<Option<StartState>>>,
    trs: Mutex<CachedData<Vec<Option<CacheTrs<W>>>>>,
    final_weights: Mutex<CachedData<Vec<Option<FinalWeight<W>>>>>,
}

#[derive(Debug, Clone)]
pub struct CacheTrs<W: Semiring> {
    pub trs: TrsVec<W>,
    pub niepsilons: usize,
    pub noepsilons: usize,
}

impl<W: Semiring> SimpleVecCache<W> {
    pub fn clear(&self) {
        let mut data_start = self.start.lock().unwrap();
        data_start.clear();

        let mut data_trs = self.trs.lock().unwrap();
        data_trs.clear();

        let mut data_final_weights = self.final_weights.lock().unwrap();
        data_final_weights.clear();
    }
}

impl<W: Semiring> Clone for SimpleVecCache<W> {
    fn clone(&self) -> Self {
        Self {
            start: Mutex::new(self.start.lock().unwrap().clone()),
            trs: Mutex::new(self.trs.lock().unwrap().clone()),
            final_weights: Mutex::new(self.final_weights.lock().unwrap().clone()),
        }
    }
}

impl<W: Semiring> Default for SimpleVecCache<W> {
    fn default() -> Self {
        Self {
            start: Mutex::new(CachedData::default()),
            trs: Mutex::new(CachedData::default()),
            final_weights: Mutex::new(CachedData::default()),
        }
    }
}

impl<W: Semiring> FstCache<W> for SimpleVecCache<W> {
    fn get_start(&self) -> Option<Option<StateId>> {
        self.start.lock().unwrap().data
    }

    fn insert_start(&self, id: Option<StateId>) {
        let mut cached_data = self.start.lock().unwrap();
        if let Some(s) = id {
            cached_data.num_known_states = std::cmp::max(cached_data.num_known_states, s + 1);
        }
        cached_data.data = Some(id);
    }

    fn get_trs(&self, id: usize) -> Option<TrsVec<W>> {
        let cached_data = self.trs.lock().unwrap();
        if id < cached_data.data.len() {
            cached_data.data[id].as_ref().map(|e| e.trs.shallow_clone())
        } else {
            None
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
        if id >= cached_data.data.len() {
            cached_data.data.resize(id + 1, None);
        }
        cached_data.data[id] = Some(CacheTrs {
            trs,
            niepsilons,
            noepsilons,
        });
    }

    fn get_final_weight(&self, id: usize) -> Option<Option<W>> {
        let cached_data = self.final_weights.lock().unwrap();
        if id < cached_data.data.len() {
            cached_data.data[id].clone()
        } else {
            // Not computed yet
            None
        }
    }

    fn insert_final_weight(&self, id: StateId, weight: Option<W>) {
        let mut cached_data = self.final_weights.lock().unwrap();
        cached_data.num_known_states = std::cmp::max(cached_data.num_known_states, id + 1);
        if id >= cached_data.data.len() {
            cached_data.data.resize(id + 1, None);
        }
        // First Some to mark the final weight as computed
        cached_data.data[id] = Some(weight);
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
        cached_data
            .data
            .get(id)
            .map(|v| v.as_ref().map(|e| e.trs.len()))
            .flatten()
    }

    fn num_input_epsilons(&self, id: usize) -> Option<usize> {
        let cached_data = self.trs.lock().unwrap();
        cached_data
            .data
            .get(id)
            .map(|v| v.as_ref().map(|e| e.niepsilons))
            .flatten()
    }

    fn num_output_epsilons(&self, id: usize) -> Option<usize> {
        let cached_data = self.trs.lock().unwrap();
        cached_data
            .data
            .get(id)
            .map(|v| v.as_ref().map(|e| e.noepsilons))
            .flatten()
    }

    fn len_trs(&self) -> usize {
        let cached_data = self.trs.lock().unwrap();
        cached_data.data.len()
    }

    fn len_final_weights(&self) -> usize {
        let cached_data = self.final_weights.lock().unwrap();
        cached_data.data.len()
    }
}
