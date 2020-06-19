use std::collections::VecDeque;
use std::iter::{repeat, Map, Repeat, Zip};
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;
use itertools::izip;
use unsafe_unwrap::UnsafeUnwrap;

use crate::algorithms::lazy_fst_revamp::fst_op::FstOp;
use crate::algorithms::lazy_fst_revamp::FstCache;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{
    AllocableFst, CoreFst, Fst, FstIterData, FstIterator, MutableFst, StateIterator,
};
use crate::semirings::Semiring;
use crate::{StateId, SymbolTable, Trs, TrsVec};

#[derive(Debug, Clone)]
pub struct LazyFst<W: Semiring, Op: FstOp<W>, Cache> {
    cache: Cache,
    pub(crate) op: Op,
    w: PhantomData<W>,
    isymt: Option<Arc<SymbolTable>>,
    osymt: Option<Arc<SymbolTable>>,
}

impl<W: Semiring, Op: FstOp<W>, Cache: FstCache<W>> CoreFst<W> for LazyFst<W, Op, Cache> {
    type TRS = TrsVec<W>;

    fn start(&self) -> Option<usize> {
        if let Some(start) = self.cache.get_start() {
            start
        } else {
            // TODO: Need to return a Result
            let start = self.op.compute_start().unwrap();
            self.cache.insert_start(start.clone());
            start
        }
    }

    fn final_weight(&self, state_id: usize) -> Result<Option<W>> {
        if let Some(final_weight) = self.cache.get_final_weight(state_id) {
            Ok(final_weight)
        } else {
            let final_weight = self.op.compute_final_weight(state_id)?;
            self.cache
                .insert_final_weight(state_id, final_weight.clone());
            Ok(final_weight)
        }
    }

    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<W> {
        self.final_weight(state_id).unsafe_unwrap()
    }

    fn num_trs(&self, s: usize) -> Result<usize> {
        self.cache
            .num_trs(s)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", s))
    }

    unsafe fn num_trs_unchecked(&self, s: usize) -> usize {
        self.cache.num_trs(s).unsafe_unwrap()
    }

    fn get_trs(&self, state_id: usize) -> Result<Self::TRS> {
        if let Some(trs) = self.cache.get_trs(state_id) {
            Ok(trs)
        } else {
            let trs = self.op.compute_trs(state_id)?;
            self.cache.insert_trs(state_id, trs.shallow_clone());
            Ok(trs)
        }
    }

    unsafe fn get_trs_unchecked(&self, state_id: usize) -> Self::TRS {
        self.get_trs(state_id).unsafe_unwrap()
    }

    fn properties(&self) -> FstProperties {
        self.op.properties()
    }

    fn num_input_epsilons(&self, state: usize) -> Result<usize> {
        self.cache
            .num_input_epsilons(state)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state))
    }

    fn num_output_epsilons(&self, state: usize) -> Result<usize> {
        self.cache
            .num_output_epsilons(state)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state))
    }
}

impl<'a, W, Op, Cache> StateIterator<'a> for LazyFst<W, Op, Cache>
where
    W: Semiring,
    Op: FstOp<W> + 'a,
    Cache: FstCache<W> + 'a,
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

impl<'a, W, Op, Cache> Iterator for StatesIteratorLazyFst<'a, LazyFst<W, Op, Cache>>
where
    W: Semiring,
    Op: FstOp<W>,
    Cache: FstCache<W>,
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

impl<'a, W, Op, Cache> FstIterator<'a, W> for LazyFst<W, Op, Cache>
where
    W: Semiring,
    Op: FstOp<W> + 'a,
    Cache: FstCache<W> + 'a,
{
    type FstIter = Map<
        Zip<<LazyFst<W, Op, Cache> as StateIterator<'a>>::Iter, Repeat<&'a Self>>,
        Box<dyn FnMut((StateId, &'a Self)) -> FstIterData<W, Self::TRS>>,
    >;

    fn fst_iter(&'a self) -> Self::FstIter {
        let it = repeat(self);
        izip!(self.states_iter(), it).map(Box::new(|(state_id, p): (StateId, &'a Self)| {
            FstIterData {
                state_id,
                trs: unsafe { p.get_trs_unchecked(state_id) },
                final_weight: unsafe { p.final_weight_unchecked(state_id) },
                num_trs: unsafe { p.num_trs_unchecked(state_id) },
            }
        }))
    }
}

impl<W, Op, Cache> Fst<W> for LazyFst<W, Op, Cache>
where
    W: Semiring,
    Op: FstOp<W> + 'static,
    Cache: FstCache<W> + 'static,
{
    fn input_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.isymt.as_ref()
    }

    fn output_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.osymt.as_ref()
    }

    fn set_input_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.isymt = Some(symt);
    }

    fn set_output_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.osymt = Some(symt);
    }

    fn take_input_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.isymt.take()
    }

    fn take_output_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.osymt.take()
    }
}

impl<W, Op, Cache> LazyFst<W, Op, Cache>
where
    W: Semiring,
    Op: FstOp<W>,
    Cache: FstCache<W>,
{
    pub(crate) fn from_op_and_cache(
        op: Op,
        cache: Cache,
        isymt: Option<Arc<SymbolTable>>,
        osymt: Option<Arc<SymbolTable>>,
    ) -> Self {
        Self {
            op,
            cache,
            isymt,
            osymt,
            w: PhantomData,
        }
    }

    /// Turns the Lazy FST into a static one.
    pub fn compute<F2: MutableFst<W> + AllocableFst<W>>(&self) -> Result<F2> {
        let start_state = self.start();
        let mut fst_out = F2::new();
        if start_state.is_none() {
            return Ok(fst_out);
        }
        let start_state = start_state.unwrap();
        fst_out.add_states(start_state + 1);
        fst_out.set_start(start_state)?;
        let mut queue = VecDeque::new();
        let mut visited_states = vec![];
        visited_states.resize(start_state + 1, false);
        visited_states[start_state] = true;
        queue.push_back(start_state);
        while !queue.is_empty() {
            let s = queue.pop_front().unwrap();
            let trs_owner = unsafe { self.get_trs_unchecked(s) };
            for tr in trs_owner.trs() {
                if tr.nextstate >= visited_states.len() {
                    visited_states.resize(tr.nextstate + 1, false);
                }
                if !visited_states[tr.nextstate] {
                    queue.push_back(tr.nextstate);
                    visited_states[tr.nextstate] = true;
                }
                let n = fst_out.num_states();
                if tr.nextstate >= n {
                    fst_out.add_states(tr.nextstate - n + 1)
                }
            }
            unsafe { fst_out.set_trs_unchecked(s, trs_owner.trs().to_vec()) };
            if let Some(f_w) = self.final_weight(s)? {
                fst_out.set_final(s, f_w)?;
            }
        }
        fst_out.set_properties(self.properties());
        // TODO: Symbol tables should be set here
        Ok(fst_out)
    }
}
