use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;

use crate::fst_traits::{CoreFst, ExpandedFst, Fst, FstIntoIterator, FstIterator, StateIterator};
use crate::semirings::Semiring;
use crate::SymbolTable;

/// Adds an object of type T to an FST.
/// The resulting type is a new FST implementation.
#[derive(Debug, PartialEq, Clone)]
pub struct FstAddOn<F, T> {
    pub(crate) fst: F,
    pub(crate) add_on: T,
}

impl<F, T> FstAddOn<F, T> {
    pub fn new(fst: F, add_on: T) -> Self {
        Self { fst, add_on }
    }

    pub fn fst(&self) -> &F {
        &self.fst
    }

    pub fn fst_mut(&mut self) -> &mut F {
        &mut self.fst
    }

    pub fn add_on(&self) -> &T {
        &self.add_on
    }
}

impl<W: Semiring, F: CoreFst<W>, T> CoreFst<W> for FstAddOn<F, T> {
    type TRS = F::TRS;

    fn start(&self) -> Option<usize> {
        self.fst.start()
    }

    fn final_weight(&self, state_id: usize) -> Result<Option<W>> {
        self.fst.final_weight(state_id)
    }

    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<W> {
        self.fst.final_weight_unchecked(state_id)
    }

    fn num_trs(&self, s: usize) -> Result<usize> {
        self.fst.num_trs(s)
    }

    unsafe fn num_trs_unchecked(&self, s: usize) -> usize {
        self.fst.num_trs_unchecked(s)
    }

    fn get_trs(&self, state_id: usize) -> Result<Self::TRS> {
        self.fst.get_trs(state_id)
    }

    unsafe fn get_trs_unchecked(&self, state_id: usize) -> Self::TRS {
        self.fst.get_trs_unchecked(state_id)
    }
}

impl<'a, F: StateIterator<'a>, T> StateIterator<'a> for FstAddOn<F, T> {
    type Iter = <F as StateIterator<'a>>::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.fst.states_iter()
    }
}

impl<'a, W, F, T> FstIterator<'a, W> for FstAddOn<F, T>
where
    W: Semiring + 'a,
    F: FstIterator<'a, W>,
{
    type FstIter = F::FstIter;

    fn fst_iter(&'a self) -> Self::FstIter {
        self.fst.fst_iter()
    }
}

impl<W, F, T: Debug> Fst<W> for FstAddOn<F, T>
where
    W: Semiring,
    F: Fst<W>,
{
    fn input_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.fst.input_symbols()
    }

    fn output_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.fst.output_symbols()
    }

    fn set_input_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.fst.set_input_symbols(symt)
    }

    fn set_output_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.fst.set_output_symbols(symt)
    }

    fn take_input_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.fst.take_input_symbols()
    }

    fn take_output_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.fst.take_output_symbols()
    }
}

impl<W, F, T> ExpandedFst<W> for FstAddOn<F, T>
where
    W: Semiring,
    F: ExpandedFst<W>,
    T: Debug + Clone + PartialEq,
{
    fn num_states(&self) -> usize {
        self.fst.num_states()
    }
}

impl<W, F, T> FstIntoIterator<W> for FstAddOn<F, T>
where
    W: Semiring,
    F: FstIntoIterator<W>,
    T: Debug,
{
    type TrsIter = F::TrsIter;
    type FstIter = F::FstIter;

    fn fst_into_iter(self) -> Self::FstIter {
        self.fst.fst_into_iter()
    }
}
