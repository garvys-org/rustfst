use std::collections::HashMap;
use std::sync::Mutex;

use anyhow::Result;

use crate::{StateId, TrsVec, Trs};
use crate::semirings::Semiring;
use crate::algorithms::lazy_fst_revamp::FstCache;

#[derive(Default, Debug)]
pub struct SimpleHashMapCache<W: Semiring> {
    // First option : has start been computed
    // Second option: value of the start state (possibly none)
    start: Mutex<Option<Option<StateId>>>,
    trs: Mutex<HashMap<StateId, TrsVec<W>>>,
    final_weight: Mutex<HashMap<StateId, Option<W>>>,
}

impl<W: Semiring> FstCache<W> for SimpleHashMapCache<W> {
    fn get_start(&self) -> Option<Option<StateId>> {
        self.start.lock().unwrap().clone()
    }

    fn insert_start(&self, id: Option<StateId>) {
        *self.start.lock().unwrap() = Some(id);
    }

    fn get_trs(&self, id: usize) -> Option<TrsVec<W>> {
        self.trs.lock().unwrap().get(&id).map(|v| v.shallow_clone())
    }

    fn insert_trs(&self, id: usize, trs: TrsVec<W>) {
        self.trs.lock().unwrap().insert(id, trs);
    }
    fn get_final_weight(&self, id: usize) -> Option<Option<W>> {
        self.final_weight.lock().unwrap().get(&id).cloned()
    }

    fn insert_final_weight(&self, id: StateId, weight: Option<W>) {
        self.final_weight.lock().unwrap().insert(id, weight);
    }

    fn num_known_states(&self) -> usize {
        std::cmp::max(self.final_weight.lock().unwrap().len(), self.trs.lock().unwrap().len())
    }
}