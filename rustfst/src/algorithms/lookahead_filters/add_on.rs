use std::fmt::Debug;
use std::rc::Rc;

use failure::Fallible;

use crate::fst_traits::{ArcIterator, CoreFst, Fst, FstIterator, StateIterator};
use crate::{Arc, SymbolTable};

/// Adds an object of type T to an FST.
/// The resulting type is a new FST implementation.
#[derive(Debug)]
pub struct FstAddOn<F, T> {
    fst: F,
    add_on: T,
}

impl<F, T> FstAddOn<F, T> {
    pub fn new(fst: F, add_on: T) -> Self {
        Self { fst, add_on }
    }

    pub fn fst(&self) -> &F {
        &self.fst
    }

    pub fn add_on(&self) -> &T {
        &self.add_on
    }
}

impl<F: CoreFst, T> CoreFst for FstAddOn<F, T> {
    type W = F::W;

    fn start(&self) -> Option<usize> {
        self.fst.start()
    }

    fn final_weight(&self, state_id: usize) -> Fallible<Option<&Self::W>> {
        self.fst.final_weight(state_id)
    }

    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<&Self::W> {
        self.fst.final_weight_unchecked(state_id)
    }

    fn num_arcs(&self, s: usize) -> Fallible<usize> {
        self.fst.num_arcs(s)
    }

    unsafe fn num_arcs_unchecked(&self, s: usize) -> usize {
        self.fst.num_arcs_unchecked(s)
    }
}

impl<'a, F: StateIterator<'a>, T> StateIterator<'a> for FstAddOn<F, T> {
    type Iter = <F as StateIterator<'a>>::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.fst.states_iter()
    }
}

impl<'a, F: ArcIterator<'a>, T> ArcIterator<'a> for FstAddOn<F, T>
where
    F::W: 'a,
{
    type Iter = <F as ArcIterator<'a>>::Iter;

    fn arcs_iter(&'a self, state_id: usize) -> Fallible<Self::Iter> {
        self.fst.arcs_iter(state_id)
    }

    unsafe fn arcs_iter_unchecked(&'a self, state_id: usize) -> Self::Iter {
        self.fst.arcs_iter_unchecked(state_id)
    }
}

impl<'a, F: FstIterator<'a>, T> FstIterator<'a> for FstAddOn<F, T>
where
    F::W: 'a,
{
    type ArcsIter = F::ArcsIter;
    type FstIter = F::FstIter;

    fn fst_iter(&'a self) -> Self::FstIter {
        self.fst.fst_iter()
    }
}

impl<F: Fst, T: Debug> Fst for FstAddOn<F, T>
where
    F::W: 'static,
{
    fn input_symbols(&self) -> Option<Rc<SymbolTable>> {
        self.fst.input_symbols()
    }

    fn output_symbols(&self) -> Option<Rc<SymbolTable>> {
        self.fst.output_symbols()
    }

    fn set_input_symbols(&mut self, symt: Rc<SymbolTable>) {
        self.fst.set_input_symbols(symt)
    }

    fn set_output_symbols(&mut self, symt: Rc<SymbolTable>) {
        self.fst.set_output_symbols(symt)
    }

    fn unset_input_symbols(&mut self) -> Option<Rc<SymbolTable>> {
        self.fst.unset_input_symbols()
    }

    fn unset_output_symbols(&mut self) -> Option<Rc<SymbolTable>> {
        self.fst.unset_output_symbols()
    }
}
