use std::fmt;
use std::sync::{Arc, Mutex};

use bimap::BiHashMap;

use crate::algorithms::determinize::{DeterminizeStateTuple, WeightedSubset};
use crate::{Semiring, StateId};
use anyhow::Result;

#[derive(Debug, PartialEq)]
struct InnerDeterminizeStateTable<W: Semiring> {
    table: BiHashMap<StateId, DeterminizeStateTuple<W>>,
    // Distance to final NFA states.
    in_dist: Option<Arc<Vec<W>>>,
    // Distance to final DFA states.
    out_dist: Vec<Option<W>>,
}

impl<W: Semiring> InnerDeterminizeStateTable<W> {
    fn compute_distance(&self, subset: &WeightedSubset<W>) -> Result<W> {
        let mut outd = W::zero();
        let weight_zero = W::zero();
        for element in subset.iter() {
            let ind = if element.state < self.in_dist.as_ref().unwrap().len() {
                &self.in_dist.as_ref().unwrap()[element.state]
            } else {
                &weight_zero
            };
            outd.plus_assign(element.weight.times(ind)?)?;
        }
        Ok(outd)
    }
}

pub struct DeterminizeStateTable<W: Semiring>(Mutex<InnerDeterminizeStateTable<W>>);

impl<W: Semiring> fmt::Debug for DeterminizeStateTable<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0.lock().unwrap())
    }
}

impl<W: Semiring> PartialEq for DeterminizeStateTable<W> {
    fn eq(&self, other: &Self) -> bool {
        self.0.lock().unwrap().eq(&*other.0.lock().unwrap())
    }
}

impl<W: Semiring> DeterminizeStateTable<W> {
    pub fn new(in_dist: Option<Arc<Vec<W>>>) -> Self {
        Self(Mutex::new(InnerDeterminizeStateTable {
            in_dist,
            out_dist: vec![],
            table: BiHashMap::new(),
        }))
    }

    /// Looks up integer ID from entry. If it doesn't exist and insert
    pub fn find_id_from_ref(&self, tuple: &DeterminizeStateTuple<W>) -> Result<StateId> {
        let mut inner = self.0.lock().unwrap();
        if !inner.table.contains_right(tuple) {
            let n = inner.table.len();
            inner.table.insert(n, tuple.clone());

            if inner.in_dist.is_some() {
                if n >= inner.out_dist.len() {
                    inner.out_dist.resize(n + 1, None);
                }
                if inner.out_dist[n].is_none() {
                    inner.out_dist[n] = Some(inner.compute_distance(&tuple.subset)?);
                }
            }

            return Ok(n);
        }

        Ok(*inner.table.get_by_right(tuple).unwrap())
    }

    /// Looks up tuple from integer ID.
    pub fn find_tuple(&self, tuple_id: StateId) -> DeterminizeStateTuple<W> {
        let inner = self.0.lock().unwrap();
        inner.table.get_by_left(&tuple_id).unwrap().clone()
    }

    pub fn out_dist(self) -> Vec<Option<W>> {
        let inner = self.0.into_inner().unwrap();
        inner.out_dist
    }
}
