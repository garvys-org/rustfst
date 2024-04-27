use std::borrow::Borrow;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::lazy::{LazyFst2, SimpleHashMapCache};
use crate::algorithms::randgen::randgen_fst_op::RandGenFstOp;
use crate::algorithms::randgen::tr_sampler::TrSampler;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{CoreFst, Fst, FstIterator, MutableFst, StateIterator};
use crate::prelude::randgen::TrSelector;
use crate::{Semiring, StateId, SymbolTable, TrsVec};

type InnerLazyFst<W, F, B, S> = LazyFst2<W, RandGenFstOp<W, F, B, S>, SimpleHashMapCache<W>>;

/// Delayed Fst sampling Fst paths through the input Fst.
pub struct RandGenFst<W: Semiring<Type = f32>, F: Fst<W>, B: Borrow<F>, S: TrSelector>(
    InnerLazyFst<W, F, B, S>,
);

impl<W, F, B, S> CoreFst<W> for RandGenFst<W, F, B, S>
where
    W: Semiring<Type = f32>,
    F: Fst<W>,
    B: Borrow<F>,
    S: TrSelector,
{
    type TRS = TrsVec<W>;

    fn start(&self) -> Option<StateId> {
        self.0.start()
    }

    fn final_weight(&self, state: StateId) -> anyhow::Result<Option<W>> {
        self.0.final_weight(state)
    }

    unsafe fn final_weight_unchecked(&self, state: StateId) -> Option<W> {
        self.0.final_weight_unchecked(state)
    }

    fn num_trs(&self, s: StateId) -> anyhow::Result<usize> {
        self.0.num_trs(s)
    }

    unsafe fn num_trs_unchecked(&self, state: StateId) -> usize {
        self.0.num_trs_unchecked(state)
    }

    fn get_trs(&self, state_id: StateId) -> anyhow::Result<Self::TRS> {
        self.0.get_trs(state_id)
    }

    unsafe fn get_trs_unchecked(&self, state: StateId) -> Self::TRS {
        self.0.get_trs_unchecked(state)
    }

    fn properties(&self) -> FstProperties {
        self.0.properties()
    }

    fn num_input_epsilons(&self, state: StateId) -> anyhow::Result<usize> {
        self.0.num_input_epsilons(state)
    }

    fn num_output_epsilons(&self, state: StateId) -> anyhow::Result<usize> {
        self.0.num_output_epsilons(state)
    }
}

impl<'a, W, F, B, S> StateIterator<'a> for RandGenFst<W, F, B, S>
where
    W: Semiring<Type = f32>,
    F: Fst<W> + 'a,
    B: Borrow<F> + 'a,
    S: TrSelector + 'a,
{
    type Iter = <InnerLazyFst<W, F, B, S> as StateIterator<'a>>::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.0.states_iter()
    }
}

impl<'a, W, F, B, S> FstIterator<'a, W> for RandGenFst<W, F, B, S>
where
    W: Semiring<Type = f32>,
    F: Fst<W> + 'a,
    B: Borrow<F> + 'a,
    S: TrSelector + 'a,
{
    type FstIter = <InnerLazyFst<W, F, B, S> as FstIterator<'a, W>>::FstIter;

    fn fst_iter(&'a self) -> Self::FstIter {
        self.0.fst_iter()
    }
}

impl<W, F, B, S> Fst<W> for RandGenFst<W, F, B, S>
where
    W: Semiring<Type = f32>,
    F: Fst<W> + 'static,
    B: Borrow<F> + 'static,
    S: TrSelector + 'static,
{
    fn input_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.0.input_symbols()
    }

    fn output_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.0.output_symbols()
    }

    fn set_input_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.0.set_input_symbols(symt)
    }

    fn set_output_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.0.set_output_symbols(symt)
    }

    fn take_input_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.0.take_input_symbols()
    }

    fn take_output_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.0.take_output_symbols()
    }
}

impl<W, F, B, S> Debug for RandGenFst<W, F, B, S>
where
    W: Semiring<Type = f32>,
    F: Fst<W> + 'static,
    B: Borrow<F> + 'static,
    S: TrSelector + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl<W, F, B, S> RandGenFst<W, F, B, S>
where
    W: Semiring<Type = f32>,
    F: Fst<W>,
    B: Borrow<F>,
    S: TrSelector,
{
    pub fn new(
        fst: B,
        sampler: TrSampler<W, F, B, S>,
        npath: usize,
        weighted: bool,
        remove_total_weight: bool,
    ) -> Self {
        let isymt = fst.borrow().input_symbols().cloned();
        let osymt = fst.borrow().output_symbols().cloned();
        let fst_op = RandGenFstOp::new(fst, sampler, npath, weighted, remove_total_weight);
        let fst_cache = SimpleHashMapCache::default();
        let lazy_fst = LazyFst2::from_op_and_cache(fst_op, fst_cache, isymt, osymt);
        RandGenFst(lazy_fst)
    }

    pub fn compute<F2: MutableFst<W>>(&self) -> Result<F2> {
        self.0.compute()
    }
}
