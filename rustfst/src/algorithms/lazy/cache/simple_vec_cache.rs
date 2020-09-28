use std::sync::Mutex;

use crate::algorithms::lazy::cache::cache_internal_types::{CachedData, FinalWeight, StartState};
use crate::algorithms::lazy::cache::fst_cache::FillableFstCache;
use crate::algorithms::lazy::{CacheStatus, FstCache};
use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::{StateId, Trs, TrsVec, EPS_LABEL};

#[derive(Debug)]
pub struct SimpleVecCache<W: Semiring> {
    // First option : has start been computed
    // Second option: value of the start state (possibly none)
    // The second element of each tuple is the number of known states.
    start: Mutex<CachedData<CacheStatus<StartState>>>,
    trs: Mutex<CachedData<Vec<CacheStatus<CacheTrs<W>>>>>,
    final_weights: Mutex<CachedData<Vec<CacheStatus<FinalWeight<W>>>>>,
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
    fn get_start(&self) -> CacheStatus<Option<StateId>> {
        self.start.lock().unwrap().data
    }

    fn insert_start(&self, id: Option<StateId>) {
        let mut cached_data = self.start.lock().unwrap();
        if let Some(s) = id {
            cached_data.num_known_states = std::cmp::max(cached_data.num_known_states, s + 1);
        }
        cached_data.data = CacheStatus::Computed(id);
    }

    fn get_trs(&self, id: usize) -> CacheStatus<TrsVec<W>> {
        let cached_data = self.trs.lock().unwrap();
        cached_data.get(id).map(|e| e.trs.shallow_clone())
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
            cached_data.data.resize(id + 1, CacheStatus::NotComputed);
        }
        cached_data.data[id] = CacheStatus::Computed(CacheTrs {
            trs,
            niepsilons,
            noepsilons,
        });
    }

    fn get_final_weight(&self, id: usize) -> CacheStatus<Option<W>> {
        let cached_data = self.final_weights.lock().unwrap();
        match cached_data.data.get(id) {
            Some(e) => e.clone(),
            None => CacheStatus::NotComputed,
        }
    }

    fn insert_final_weight(&self, id: StateId, weight: Option<W>) {
        let mut cached_data = self.final_weights.lock().unwrap();
        cached_data.num_known_states = std::cmp::max(cached_data.num_known_states, id + 1);
        if id >= cached_data.data.len() {
            cached_data.data.resize(id + 1, CacheStatus::NotComputed);
        }
        // First Some to mark the final weight as computed
        cached_data.data[id] = CacheStatus::Computed(weight);
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
        cached_data.get(id).map(|e| e.trs.len()).into_option()
    }

    fn num_input_epsilons(&self, id: usize) -> CacheStatus<usize> {
        let cached_data = self.trs.lock().unwrap();
        cached_data.get(id).map(|e| e.niepsilons)
    }

    unsafe fn num_input_epsilons_unchecked(&self, id: usize) -> usize {
        let cached_data = self.trs.lock().unwrap();
        cached_data.get_unchecked(id).niepsilons
    }

    fn num_output_epsilons(&self, id: usize) -> CacheStatus<usize> {
        let cached_data = self.trs.lock().unwrap();
        cached_data.get(id).map(|e| e.noepsilons)
    }

    unsafe fn num_output_epsilons_unchecked(&self, id: usize) -> usize {
        let cached_data = self.trs.lock().unwrap();
        cached_data.get_unchecked(id).noepsilons
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
        let cached_data = self.final_weights.lock().unwrap();
        match cached_data.data.get(state_id) {
            Some(e) => match e {
                CacheStatus::Computed(v) => CacheStatus::Computed(v.is_some()),
                CacheStatus::NotComputed => CacheStatus::NotComputed,
            },
            None => CacheStatus::NotComputed,
        }
    }

    unsafe fn is_final_unchecked(&self, state_id: usize) -> bool {
        let cached_data = self.final_weights.lock().unwrap();
        match cached_data.data.get_unchecked(state_id) {
            CacheStatus::Computed(e) => e.is_some(),
            CacheStatus::NotComputed => unreachable!(),
        }
    }
}

impl<W: Semiring> FillableFstCache<W> for SimpleVecCache<W> {
    fn into_fst<F: MutableFst<W>>(self) -> F {
        let mut fst_out = F::new();

        // Safe because computed
        if let Some(start) = self.get_start().unwrap() {
            let nstates = self.num_known_states();
            fst_out.add_states(nstates);

            unsafe { fst_out.set_start_unchecked(start) };

            let final_weights = self.final_weights.into_inner().unwrap().data;
            let trs = self.trs.into_inner().unwrap().data;

            for (state_id, cache_trs) in trs.into_iter().enumerate() {
                let cache_trs = cache_trs.unwrap();
                unsafe {
                    fst_out.set_state_unchecked_noprops(
                        state_id,
                        cache_trs.trs,
                        cache_trs.niepsilons,
                        cache_trs.noepsilons,
                    )
                };
            }

            for (state_id, final_weight) in final_weights.into_iter().enumerate() {
                // Safe as computed
                if let Some(final_weight) = final_weight.unwrap() {
                    unsafe { fst_out.set_final_unchecked(state_id, final_weight) };
                }
            }
        }

        fst_out
    }
}
