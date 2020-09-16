use std::sync::Mutex;

use crate::algorithms::lazy_fst_revamp::FstCache;
use crate::semirings::Semiring;
use crate::{StateId, Trs, TrsVec, EPS_LABEL};

#[derive(Debug)]
pub struct SimpleVecCache<W: Semiring> {
    // First option : has start been computed
    // Second option: value of the start state (possibly none)
    // The second element of each tuple is the number of known states.
    start: Mutex<(Option<Option<StateId>>, usize)>,
    trs: Mutex<(Vec<Option<CacheTrs<W>>>, usize)>,
    final_weights: Mutex<(Vec<Option<Option<W>>>, usize)>,
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
            start: Mutex::new((None, 0)),
            trs: Mutex::new((Vec::new(), 0)),
            final_weights: Mutex::new((Vec::new(), 0)),
        }
    }
}

impl<W: Semiring> FstCache<W> for SimpleVecCache<W> {
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
        let data = self.trs.lock().unwrap();
        if id < data.0.len() {
            data.0[id].as_ref().map(|e| e.trs.shallow_clone())
        } else {
            None
        }
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
        if id >= data.0.len() {
            data.0.resize(id + 1, None);
        }
        data.0[id] = Some(CacheTrs {
            trs,
            niepsilons,
            noepsilons,
        });
    }

    fn get_final_weight(&self, id: usize) -> Option<Option<W>> {
        let data = self.final_weights.lock().unwrap();
        if id < data.0.len() {
            data.0[id].clone()
        } else {
            // Not computed yet
            None
        }
    }

    fn insert_final_weight(&self, id: StateId, weight: Option<W>) {
        let mut data = self.final_weights.lock().unwrap();
        data.1 = std::cmp::max(data.1, id + 1);
        if id >= data.0.len() {
            data.0.resize(id + 1, None);
        }
        // First Some to mark the final weight as computed
        data.0[id] = Some(weight);
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
        data.0
            .get(id)
            .map(|v| v.as_ref().map(|e| e.trs.len()))
            .flatten()
    }

    fn num_input_epsilons(&self, id: usize) -> Option<usize> {
        let data = self.trs.lock().unwrap();
        data.0
            .get(id)
            .map(|v| v.as_ref().map(|e| e.niepsilons))
            .flatten()
    }

    fn num_output_epsilons(&self, id: usize) -> Option<usize> {
        let data = self.trs.lock().unwrap();
        data.0
            .get(id)
            .map(|v| v.as_ref().map(|e| e.noepsilons))
            .flatten()
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
