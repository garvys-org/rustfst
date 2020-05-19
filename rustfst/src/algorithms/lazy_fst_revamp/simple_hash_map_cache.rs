use std::collections::HashMap;
use std::sync::Mutex;

use crate::algorithms::lazy_fst_revamp::FstCache;
use crate::semirings::Semiring;
use crate::{StateId, Trs, TrsVec};

#[derive(Default, Debug)]
pub struct SimpleHashMapCache<W: Semiring> {
    // First option : has start been computed
    // Second option: value of the start state (possibly none)
    start: Mutex<Option<Option<StateId>>>,
    trs: Mutex<HashMap<StateId, TrsVec<W>>>,
    final_weight: Mutex<HashMap<StateId, Option<W>>>,
    num_known_states: Mutex<usize>
}

impl<W: Semiring> SimpleHashMapCache<W> {
    pub fn new() -> Self {
        Self {
            start: Mutex::new(None),
            trs: Mutex::new(HashMap::new()),
            final_weight: Mutex::new(HashMap::new()),
            num_known_states: Mutex::new(0)
        }
    }
}

impl<W: Semiring> FstCache<W> for SimpleHashMapCache<W> {
    fn get_start(&self) -> Option<Option<StateId>> {
        self.start.lock().unwrap().clone()
    }

    fn insert_start(&self, id: Option<StateId>)
    {
        if let Some(s) = id {
            let mut n = self.num_known_states.lock().unwrap();
            *n = std::cmp::max(*n, s+1);
        }
        *self.start.lock().unwrap() = Some(id);
    }

    fn get_trs(&self, id: usize) -> Option<TrsVec<W>> {
        self.trs.lock().unwrap().get(&id).map(|v| v.shallow_clone())
    }

    fn insert_trs(&self, id: usize, trs: TrsVec<W>) {
        let mut n = self.num_known_states.lock().unwrap();
        *n = std::cmp::max(*n, id+1);
        for tr in trs.trs() {
            *n = std::cmp::max(*n, tr.nextstate+1);
        }
        drop(n);
        self.trs.lock().unwrap().insert(id, trs);
    }
    fn get_final_weight(&self, id: usize) -> Option<Option<W>> {
        self.final_weight.lock().unwrap().get(&id).cloned()
    }

    fn insert_final_weight(&self, id: StateId, weight: Option<W>) {
        let mut n = self.num_known_states.lock().unwrap();
        *n = std::cmp::max(*n, id+1);
        drop(n);
        self.final_weight.lock().unwrap().insert(id, weight);
    }

    fn num_known_states(&self) -> usize {
        *self.num_known_states.lock().unwrap()
    }
}
