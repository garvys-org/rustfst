use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::fst_traits::{CoreFst, Fst};
use crate::semirings::Semiring;
use crate::{Trs, StateId};
use unsafe_unwrap::UnsafeUnwrap;

trait FstCache<W: Semiring, T> {
    fn get_start(&self) -> Option<Option<StateId>>;
    fn insert_start(&self, id: Option<StateId>);
    fn get_trs(&self, id: StateId) -> Option<T>;
    fn insert_trs(&self, id: StateId, trs: T);
    fn get_final_weight(&self, id: StateId) -> Option<Option<W>>;
    fn insert_final_weight(&self, id: StateId, weight: Option<W>);
}

trait FstOp<W: Semiring, T> {
    // was FstImpl
    fn compute_start(&self) -> Option<StateId>;
    fn compute_trs(&self, fst: &dyn CoreFst<W, TRS=T>, id: usize) -> T;
    fn compute_final_weight(&self, id: StateId) -> Option<W>;
}

#[derive(Default)]
struct SimpleHashMapCache<W: Semiring, T: Trs<W>> {
    // First option : has start been computed
    // Second option: value of the start state (possibly none)
    start: Mutex<Option<Option<StateId>>>,
    trs: Mutex<HashMap<StateId, T>>,
    final_weight: Mutex<HashMap<StateId, Option<W>>>,
}

impl<W: Semiring, T: Trs<W>> FstCache<W, T> for SimpleHashMapCache<W, T> {
    fn get_start(&self) -> Option<Option<StateId>> {
        self.start.lock().unwrap().clone()
    }

    fn insert_start(&self, id: Option<StateId>) {
        *self.start.lock().unwrap() = Some(id);
    }

    fn get_trs(&self, id: usize) -> Option<T> {
        self.trs.lock().unwrap().get(&id).map(|v| v.shallow_clone())
    }

    fn insert_trs(&self, id: usize, trs: T) {
        self.trs.lock().unwrap().insert(id, trs);
    }
    fn get_final_weight(&self, id: usize) -> Option<Option<W>> {
        self.final_weight.lock().unwrap().get(&id).cloned()
    }

    fn insert_final_weight(&self, id: StateId, weight: Option<W>) {
        self.final_weight.lock().unwrap().insert(id, weight);
    }
}

struct LazyFST2<W: Semiring, T: Trs<W>, Op: FstOp<W, T>, Cache: FstCache<W, T>> {
    cache: Cache,
    op: Op,
    w: PhantomData<W>,
    t: PhantomData<T>
}

impl<W: Semiring, T: Trs<W>, Op: FstOp<W, T>, Cache: FstCache<W, T>> CoreFst<W> for LazyFST2<W, T, Op, Cache> {
    type TRS = T;

    fn start(&self) -> Option<usize> {
         self.cache.get_start().unwrap()
    }

    fn final_weight(&self, state_id: usize) -> Result<Option<&W>> {
        unimplemented!()
    }

    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<&W> {
        unimplemented!()
    }

    fn num_trs(&self, s: usize) -> Result<usize> {
        let c = self.cache.get_trs(s).ok_or_else(|| format_err!("State {} not present in cache", s))?;
        Ok(c.len())
    }

    unsafe fn num_trs_unchecked(&self, s: usize) -> usize {
        let c = self.cache.get_trs(s).unsafe_unwrap();
        c.len()
    }

    fn get_trs(&self, state_id: usize) -> Result<Self::TRS> {
        let c = self.cache.get_trs(state_id).ok_or_else(|| format_err!("Trs for state {} missing in cache", state_id))?;
        Ok(c)
    }

    unsafe fn get_trs_unchecked(&self, state_id: usize) -> Self::TRS {
        self.cache.get_trs(state_id).unsafe_unwrap()
    }
}