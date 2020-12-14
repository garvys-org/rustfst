use std::borrow::Borrow;
use std::fmt;
use std::sync::Mutex;

use bimap::BiHashMap;

use crate::algorithms::determinize::{DeterminizeStateTuple, WeightedSubset};
use crate::{Semiring, StateId};
use anyhow::Result;

#[derive(Debug, PartialEq)]
struct InnerDeterminizeStateTable<W: Semiring, B: Borrow<[W]>> {
    table: BiHashMap<StateId, DeterminizeStateTuple<W>>,
    // Distance to final NFA states.
    in_dist: Option<B>,
    // Distance to final DFA states.
    out_dist: Vec<Option<W>>,
}

impl<W: Semiring, B: Borrow<[W]> + PartialEq> InnerDeterminizeStateTable<W, B> {
    fn compute_distance(&self, subset: &WeightedSubset<W>) -> Result<W> {
        let mut outd = W::zero();
        let weight_zero = W::zero();
        for element in subset.iter() {
            let ind = self
                .in_dist
                .as_ref()
                .unwrap()
                .borrow()
                .get(element.state as usize)
                .unwrap_or(&weight_zero);
            outd.plus_assign(element.weight.times(ind)?)?;
        }
        Ok(outd)
    }
}

pub struct DeterminizeStateTable<W: Semiring, B: Borrow<[W]>>(
    Mutex<InnerDeterminizeStateTable<W, B>>,
);

impl<W: Semiring, B: Borrow<[W]> + fmt::Debug> fmt::Debug for DeterminizeStateTable<W, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0.lock().unwrap())
    }
}

impl<W: Semiring, B: Borrow<[W]> + PartialEq> PartialEq for DeterminizeStateTable<W, B> {
    fn eq(&self, other: &Self) -> bool {
        self.0.lock().unwrap().eq(&*other.0.lock().unwrap())
    }
}

impl<W: Semiring, B: Borrow<[W]>> DeterminizeStateTable<W, B> {
    pub fn new(in_dist: Option<B>) -> Self {
        Self(Mutex::new(InnerDeterminizeStateTable {
            in_dist,
            out_dist: vec![],
            table: BiHashMap::new(),
        }))
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

impl<W: Semiring, B: Borrow<[W]> + PartialEq> DeterminizeStateTable<W, B> {
    /// Looks up integer ID from entry. If it doesn't exist and insert
    pub fn find_id_from_ref(&self, tuple: &DeterminizeStateTuple<W>) -> Result<StateId> {
        let mut inner = self.0.lock().unwrap();
        if !inner.table.contains_right(tuple) {
            let n = inner.table.len();
            inner.table.insert(n as StateId, tuple.clone());

            if inner.in_dist.is_some() {
                if n >= inner.out_dist.len() {
                    inner.out_dist.resize(n + 1, None);
                }
                if inner.out_dist[n].is_none() {
                    inner.out_dist[n] = Some(inner.compute_distance(&tuple.subset)?);
                }
            }

            return Ok(n as StateId);
        }

        Ok(*inner.table.get_by_right(tuple).unwrap())
    }
}
