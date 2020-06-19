use std::collections::HashMap;
use std::sync::Mutex;

use crate::algorithms::lazy_fst_revamp::FstCache;
use crate::semirings::Semiring;
use crate::{StateId, Trs, TrsVec, EPS_LABEL};

#[derive(Debug)]
pub struct SimpleHashMapCache<W: Semiring> {
    // First option : has start been computed
    // Second option: value of the start state (possibly none)
    // The second element of each tuple is the number of known states.
    start: Mutex<(Option<Option<StateId>>, usize)>,
    trs: Mutex<(HashMap<StateId, CacheTrs<W>>, usize)>,
    final_weights: Mutex<(HashMap<StateId, Option<W>>, usize)>,
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
        data_start.0.take();
        data_start.1 = 0;

        let mut data_trs = self.trs.lock().unwrap();
        data_trs.0.clear();
        data_trs.1 = 0;

        let mut data_final_weights = self.final_weights.lock().unwrap();
        data_final_weights.0.clear();
        data_final_weights.1 = 0;
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
                start: Mutex::new((None, 0)),
                trs: Mutex::new((HashMap::new(), 0)),
                final_weights: Mutex::new((HashMap::new(), 0)),}
    }
}

impl<W: Semiring> FstCache<W> for SimpleHashMapCache<W> {
    fn get_start(&self) -> Option<Option<StateId>> {
        self.start.lock().unwrap().0
    }

    fn insert_start(&self, id: Option<StateId>) {
        let mut data = self.start.lock().unwrap();
        if let Some(s) = id {
            data.1 = std::cmp::max(data.1, s + 1);
        }
        data.0 = Some(id);
    }

    fn get_trs(&self, id: usize) -> Option<TrsVec<W>> {
        self.trs
            .lock()
            .unwrap()
            .0
            .get(&id)
            .map(|v| v.trs.shallow_clone())
    }

    fn insert_trs(&self, id: usize, trs: TrsVec<W>) {
        let mut data = self.trs.lock().unwrap();
        let mut niepsilons = 0;
        let mut noepsilons = 0;
        for tr in trs.trs() {
            data.1 = std::cmp::max(data.1, tr.nextstate + 1);
            if tr.ilabel == EPS_LABEL {
                niepsilons += 1;
            }
            if tr.olabel == EPS_LABEL {
                noepsilons += 1;
            }
        }
        data.0.insert(
            id,
            CacheTrs {
                trs,
                niepsilons,
                noepsilons,
            },
        );
    }
    fn get_final_weight(&self, id: usize) -> Option<Option<W>> {
        self.final_weights.lock().unwrap().0.get(&id).cloned()
    }

    fn insert_final_weight(&self, id: StateId, weight: Option<W>) {
        let mut data = self.final_weights.lock().unwrap();
        data.1 = std::cmp::max(data.1, id + 1);
        data.0.insert(id, weight);
    }

    fn num_known_states(&self) -> usize {
        let mut n = 0;
        n = std::cmp::max(n, self.start.lock().unwrap().1);
        n = std::cmp::max(n, self.trs.lock().unwrap().1);
        n = std::cmp::max(n, self.final_weights.lock().unwrap().1);
        n
    }

    fn num_trs(&self, id: usize) -> Option<usize> {
        let data = self.trs.lock().unwrap();
        data.0.get(&id).map(|v| v.trs.len())
    }

    fn num_input_epsilons(&self, id: usize) -> Option<usize> {
        let data = self.trs.lock().unwrap();
        data.0.get(&id).map(|v| v.niepsilons)
    }

    fn num_output_epsilons(&self, id: usize) -> Option<usize> {
        let data = self.trs.lock().unwrap();
        data.0.get(&id).map(|v| v.noepsilons)
    }

    fn len_trs(&self) -> usize {
        let data = self.trs.lock().unwrap();
        data.0.len()
    }

    fn len_final_weights(&self) -> usize {
        let data = self.final_weights.lock().unwrap();
        data.0.len()
    }
}
