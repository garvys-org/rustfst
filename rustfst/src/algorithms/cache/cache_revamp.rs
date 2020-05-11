use std::collections::HashMap;
use std::iter::{Map, Repeat, repeat, Zip};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use itertools::izip;
use unsafe_unwrap::UnsafeUnwrap;

use crate::{StateId, Trs};
use crate::fst_traits::{CoreFst, Fst, FstIterator, FstIterData, StateIterator};
use crate::semirings::Semiring;

trait FstCache<W: Semiring, T: Trs<W>> {
    fn get_start(&self) -> Option<Option<StateId>>;
    fn insert_start(&self, id: Option<StateId>);

    fn get_trs(&self, id: StateId) -> Option<T>;
    fn insert_trs(&self, id: StateId, trs: T);

    fn get_final_weight(&self, id: StateId) -> Option<Option<W>>;
    fn insert_final_weight(&self, id: StateId, weight: Option<W>);

    fn num_known_states(&self) -> usize;
}

trait FstOp<W: Semiring, T: Trs<W>> {
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

    fn num_known_states(&self) -> usize {
        std::cmp::max(self.final_weight.lock().unwrap().len(), self.trs.lock().unwrap().len())
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
         if let Some(start) = self.cache.get_start() {
             start
         } else {
             let start = self.op.compute_start();
             self.cache.insert_start(start.clone());
             start
         }
    }

    fn final_weight(&self, state_id: usize) -> Result<Option<W>> {
        if let Some(final_weight) = self.cache.get_final_weight(state_id) {
            Ok(final_weight)
        } else {
            let final_weight = self.op.compute_final_weight(state_id);
            self.cache.insert_final_weight(state_id, final_weight.clone());
            Ok(final_weight)
        }
    }

    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<W> {
        self.final_weight(state_id).unsafe_unwrap()
    }

    fn get_trs(&self, state_id: usize) -> Result<Self::TRS> {
        if let Some(trs) = self.cache.get_trs(state_id) {
            Ok(trs)
        } else {
            let trs = self.op.compute_trs(self, state_id);
            self.cache.insert_trs(state_id, trs.shallow_clone());
            Ok(trs)
        }
    }

    unsafe fn get_trs_unchecked(&self, state_id: usize) -> Self::TRS {
        self.get_trs(state_id).unsafe_unwrap()
    }
}

impl<'a, W, T, Op, Cache> StateIterator<'a> for LazyFST2<W, T, Op, Cache>
where
    W: Semiring,
    T: Trs<W> + 'a,
    Op: FstOp<W, T> + 'a,
    Cache: FstCache<W, T> + 'a
{
    type Iter = StatesIteratorLazyFst<'a, Self>;

    fn states_iter(&'a self) -> Self::Iter {
        self.start();
        StatesIteratorLazyFst { fst: &self, s: 0 }
    }
}

#[derive(Clone)]
pub struct StatesIteratorLazyFst<'a, T> {
    pub(crate) fst: &'a T,
    pub(crate) s: usize,
}

impl<'a, W, T, Op, Cache> Iterator for StatesIteratorLazyFst<'a, LazyFST2<W, T, Op, Cache>>
where
    W: Semiring,
    T: Trs<W>,
    Op: FstOp<W, T>,
    Cache: FstCache<W, T>
{
    type Item = StateId;

    fn next(&mut self) -> Option<Self::Item> {
        let num_known_states = self.fst.cache.num_known_states();
        if self.s < num_known_states {
            let s_cur = self.s;
            // Force expansion of the state
            self.fst.get_trs(self.s).unwrap();
            self.s += 1;
            Some(s_cur)
        } else {
            None
        }
    }
}

impl<'a, W, T, Op, Cache> FstIterator<'a, W> for LazyFST2<W, T, Op, Cache>
    where
        W: Semiring,
        T: Trs<W> + 'a,
        Op: FstOp<W, T> + 'a,
        Cache: FstCache<W, T> + 'a
{
    type FstIter = Map<
        Zip<<LazyFST2<W, T, Op, Cache> as StateIterator<'a>>::Iter, Repeat<&'a Self>>,
        Box<dyn FnMut((StateId, &'a Self)) -> FstIterData<W, Self::TRS>>,
    >;

    fn fst_iter(&'a self) -> Self::FstIter {
        let it = repeat(self);
        izip!(self.states_iter(), it).map(Box::new(|(state_id, p): (StateId, &'a Self)| {
            FstIterData {
                state_id,
                trs: unsafe {p.get_trs_unchecked(state_id)},
                final_weight: unsafe {p.final_weight_unchecked(state_id)},
                num_trs: unsafe {p.num_trs_unchecked(state_id)},
            }
        }))
    }
}
