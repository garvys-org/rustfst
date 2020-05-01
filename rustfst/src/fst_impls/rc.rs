use std::ops::Deref;
use std::rc::Rc;
use std::sync;

use anyhow::Result;

use crate::fst_traits::{
    ArcIterator, CoreFst, ExpandedFst, Fst, FstIntoIterator, FstIterator, StateIterator,
};
use crate::SymbolTable;

impl<F: Fst> Fst for Rc<F>
where
    F::W: 'static,
{
    fn input_symbols(&self) -> Option<sync::Arc<SymbolTable>> {
        self.deref().input_symbols()
    }

    fn output_symbols(&self) -> Option<sync::Arc<SymbolTable>> {
        self.deref().output_symbols()
    }

    fn set_input_symbols(&mut self, _symt: sync::Arc<SymbolTable>) {
        unimplemented!()
    }

    fn set_output_symbols(&mut self, _symt: sync::Arc<SymbolTable>) {
        unimplemented!()
    }

    fn unset_input_symbols(&mut self) -> Option<sync::Arc<SymbolTable>> {
        unimplemented!()
    }

    fn unset_output_symbols(&mut self) -> Option<sync::Arc<SymbolTable>> {
        unimplemented!()
    }
}

impl<F: ExpandedFst> ExpandedFst for Rc<F>
where
    F::W: 'static,
{
    fn num_states(&self) -> usize {
        self.deref().num_states()
    }
}

impl<F: CoreFst> CoreFst for Rc<F> {
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

    fn num_arcs(&self, s: usize) -> Result<usize> {
        self.deref().num_arcs(s)
    }

    unsafe fn num_arcs_unchecked(&self, s: usize) -> usize {
        self.deref().num_arcs_unchecked(s)
    }
}

impl<'a, F: FstIterator<'a>> FstIterator<'a> for Rc<F>
where
    F::W: 'a,
{
    type ArcsIter = F::ArcsIter;
    type FstIter = F::FstIter;

    fn fst_iter(&'a self) -> Self::FstIter {
        self.deref().fst_iter()
    }
}

impl<'a, F: ArcIterator<'a>> ArcIterator<'a> for Rc<F>
where
    F::W: 'a,
{
    type Iter = F::Iter;

    fn arcs_iter(&'a self, state_id: usize) -> Result<Self::Iter> {
        self.deref().arcs_iter(state_id)
    }

    unsafe fn arcs_iter_unchecked(&'a self, state_id: usize) -> Self::Iter {
        self.deref().arcs_iter_unchecked(state_id)
    }
}

impl<'a, F: StateIterator<'a>> StateIterator<'a> for Rc<F> {
    type Iter = F::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.deref().states_iter()
    }
}

impl<F: FstIntoIterator> FstIntoIterator for Rc<F> {
    type ArcsIter = F::ArcsIter;
    type FstIter = F::FstIter;

    fn fst_into_iter(self) -> Self::FstIter {
        unimplemented!()
    }
}



impl<F: Fst> Fst for sync::Arc<F>
where
    F::W: 'static,
{
    fn input_symbols(&self) -> Option<sync::Arc<SymbolTable>> {
        self.deref().input_symbols()
    }

    fn output_symbols(&self) -> Option<sync::Arc<SymbolTable>> {
        self.deref().output_symbols()
    }

    fn set_input_symbols(&mut self, _symt: sync::Arc<SymbolTable>) {
        unimplemented!()
    }

    fn set_output_symbols(&mut self, _symt: sync::Arc<SymbolTable>) {
        unimplemented!()
    }

    fn unset_input_symbols(&mut self) -> Option<sync::Arc<SymbolTable>> {
        unimplemented!()
    }

    fn unset_output_symbols(&mut self) -> Option<sync::Arc<SymbolTable>> {
        unimplemented!()
    }
}

impl<F: ExpandedFst> ExpandedFst for sync::Arc<F>
where
    F::W: 'static,
{
    fn num_states(&self) -> usize {
        self.deref().num_states()
    }
}

impl<F: CoreFst> CoreFst for sync::Arc<F> {
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

    fn num_arcs(&self, s: usize) -> Result<usize> {
        self.deref().num_arcs(s)
    }

    unsafe fn num_arcs_unchecked(&self, s: usize) -> usize {
        self.deref().num_arcs_unchecked(s)
    }
}

impl<'a, F: FstIterator<'a>> FstIterator<'a> for sync::Arc<F>
where
    F::W: 'a,
{
    type ArcsIter = F::ArcsIter;
    type FstIter = F::FstIter;

    fn fst_iter(&'a self) -> Self::FstIter {
        self.deref().fst_iter()
    }
}

impl<'a, F: ArcIterator<'a>> ArcIterator<'a> for sync::Arc<F>
where
    F::W: 'a,
{
    type Iter = F::Iter;

    fn arcs_iter(&'a self, state_id: usize) -> Result<Self::Iter> {
        self.deref().arcs_iter(state_id)
    }

    unsafe fn arcs_iter_unchecked(&'a self, state_id: usize) -> Self::Iter {
        self.deref().arcs_iter_unchecked(state_id)
    }
}

impl<'a, F: StateIterator<'a>> StateIterator<'a> for sync::Arc<F> {
    type Iter = F::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.deref().states_iter()
    }
}

impl<F: FstIntoIterator> FstIntoIterator for sync::Arc<F> {
    type ArcsIter = F::ArcsIter;
    type FstIter = F::FstIter;

    fn fst_into_iter(self) -> Self::FstIter {
        unimplemented!()
    }
}
