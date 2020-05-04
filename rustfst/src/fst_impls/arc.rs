use std::ops::Deref;
use std::sync::Arc;

use anyhow::Result;

use crate::fst_traits::{
    CoreFst, ExpandedFst, Fst, FstIntoIterator, FstIterator, StateIterator, TrIterator,
};
use crate::SymbolTable;

impl<F: Fst> Fst for Arc<F>
where
    F::W: 'static,
{
    fn input_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.deref().input_symbols()
    }

    fn output_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.deref().output_symbols()
    }

    fn set_input_symbols(&mut self, _symt: Arc<SymbolTable>) {
        unimplemented!()
    }

    fn set_output_symbols(&mut self, _symt: Arc<SymbolTable>) {
        unimplemented!()
    }

    fn take_input_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        unimplemented!()
    }

    fn take_output_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        unimplemented!()
    }
}

impl<F: ExpandedFst> ExpandedFst for Arc<F>
where
    F::W: 'static,
{
    fn num_states(&self) -> usize {
        self.deref().num_states()
    }
}

impl<F: CoreFst> CoreFst for Arc<F> {
    type W = F::W;

    fn start(&self) -> Option<usize> {
        self.deref().start()
    }

    fn final_weight(&self, state_id: usize) -> Result<Option<&Self::W>> {
        self.deref().final_weight(state_id)
    }

    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<&Self::W> {
        self.deref().final_weight_unchecked(state_id)
    }

    fn num_trs(&self, s: usize) -> Result<usize> {
        self.deref().num_trs(s)
    }

    unsafe fn num_trs_unchecked(&self, s: usize) -> usize {
        self.deref().num_trs_unchecked(s)
    }
}

impl<'a, F: FstIterator<'a>> FstIterator<'a> for Arc<F>
where
    F::W: 'a,
{
    type TrsIter = F::TrsIter;
    type FstIter = F::FstIter;

    fn fst_iter(&'a self) -> Self::FstIter {
        self.deref().fst_iter()
    }
}

impl<'a, F: TrIterator<'a>> TrIterator<'a> for Arc<F>
where
    F::W: 'a,
{
    type Iter = F::Iter;

    fn tr_iter(&'a self, state_id: usize) -> Result<Self::Iter> {
        self.deref().tr_iter(state_id)
    }

    unsafe fn tr_iter_unchecked(&'a self, state_id: usize) -> Self::Iter {
        self.deref().tr_iter_unchecked(state_id)
    }
}

impl<'a, F: StateIterator<'a>> StateIterator<'a> for Arc<F> {
    type Iter = F::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.deref().states_iter()
    }
}

impl<F: FstIntoIterator> FstIntoIterator for Arc<F> {
    type TrsIter = F::TrsIter;
    type FstIter = F::FstIter;

    fn fst_into_iter(self) -> Self::FstIter {
        unimplemented!()
    }
}
