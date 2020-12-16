use crate::algorithms::lazy::{CacheStatus, FstCache};
use crate::{Semiring, StateId, Trs, TrsVec};
use std::sync::Mutex;

#[derive(Debug)]
pub struct FirstCache<W: Semiring, Cache: FstCache<W>> {
    cache: Cache,
    last_trs: Mutex<Option<(StateId, TrsVec<W>)>>,
    last_final_weight: Mutex<Option<(StateId, Option<W>)>>,
}

impl<W: Semiring, Cache: FstCache<W> + Default> Default for FirstCache<W, Cache> {
    fn default() -> Self {
        Self {
            cache: Cache::default(),
            last_trs: Mutex::new(None),
            last_final_weight: Mutex::new(None),
        }
    }
}

impl<W: Semiring, Cache: FstCache<W>> FstCache<W> for FirstCache<W, Cache> {
    fn get_start(&self) -> CacheStatus<Option<StateId>> {
        self.cache.get_start()
    }

    fn insert_start(&self, id: Option<StateId>) {
        self.cache.insert_start(id)
    }

    fn get_trs(&self, id: StateId) -> CacheStatus<TrsVec<W>> {
        let data = self.last_trs.lock().unwrap();
        if let Some((last_id_trs, last_trs)) = &*data {
            if *last_id_trs == id {
                return CacheStatus::Computed(last_trs.shallow_clone());
            }
        }
        self.cache.get_trs(id)
    }

    fn insert_trs(&self, id: StateId, trs: TrsVec<W>) {
        let mut data = self.last_trs.lock().unwrap();
        *data = Some((id, trs.shallow_clone()));
        self.cache.insert_trs(id, trs);
    }

    fn get_final_weight(&self, id: StateId) -> CacheStatus<Option<W>> {
        let data = self.last_final_weight.lock().unwrap();
        if let Some((last_id_final_weight, last_final_weight)) = &*data {
            if *last_id_final_weight == id {
                return CacheStatus::Computed(last_final_weight.clone());
            }
        }
        self.cache.get_final_weight(id)
    }

    fn insert_final_weight(&self, id: StateId, weight: Option<W>) {
        let mut data = self.last_final_weight.lock().unwrap();
        *data = Some((id, weight.clone()));
        self.cache.insert_final_weight(id, weight)
    }

    fn num_known_states(&self) -> usize {
        self.cache.num_known_states()
    }

    fn compute_num_known_trs(&self) -> usize {
        self.cache.compute_num_known_trs()
    }

    fn num_trs(&self, id: StateId) -> Option<usize> {
        self.cache.num_trs(id)
    }

    fn num_input_epsilons(&self, id: StateId) -> Option<usize> {
        self.cache.num_input_epsilons(id)
    }

    fn num_output_epsilons(&self, id: StateId) -> Option<usize> {
        self.cache.num_output_epsilons(id)
    }

    fn len_trs(&self) -> usize {
        self.cache.len_trs()
    }

    fn len_final_weights(&self) -> usize {
        self.cache.len_final_weights()
    }
}
