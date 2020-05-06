use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;

use crate::fst_traits::{
    CoreFst, ExpandedFst, Fst, FstIntoIterator, FstIterator, StateIterator,
};
use crate::SymbolTable;

/// Adds an object of type T to an FST.
/// The resulting type is a new FST implementation.
#[derive(Debug, PartialEq, Clone)]
pub struct FstAddOn<F, T> {
    pub(crate) fst: F,
    pub(crate) add_on: T,
}

// impl<F, T> FstAddOn<F, T> {
//     pub fn new(fst: F, add_on: T) -> Self {
//         Self { fst, add_on }
//     }
//
//     pub fn fst(&self) -> &F {
//         &self.fst
//     }
//
//     pub fn fst_mut(&mut self) -> &mut F {
//         &mut self.fst
//     }
//
//     pub fn add_on(&self) -> &T {
//         &self.add_on
//     }
// }
//
// impl<F: CoreFst, T> CoreFst for FstAddOn<F, T> {
//     type W = F::W;
//
//     fn start(&self) -> Option<usize> {
//         self.fst.start()
//     }
//
//     fn final_weight(&self, state_id: usize) -> Result<Option<&Self::W>> {
//         self.fst.final_weight(state_id)
//     }
//
//     unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<&Self::W> {
//         self.fst.final_weight_unchecked(state_id)
//     }
//
//     fn num_trs(&self, s: usize) -> Result<usize> {
//         self.fst.num_trs(s)
//     }
//
//     unsafe fn num_trs_unchecked(&self, s: usize) -> usize {
//         self.fst.num_trs_unchecked(s)
//     }
// }
//
// impl<'a, F: StateIterator<'a>, T> StateIterator<'a> for FstAddOn<F, T> {
//     type Iter = <F as StateIterator<'a>>::Iter;
//
//     fn states_iter(&'a self) -> Self::Iter {
//         self.fst.states_iter()
//     }
// }
//
// impl<'a, F: TrIterator<'a>, T> TrIterator<'a> for FstAddOn<F, T>
// where
//     F::W: 'a,
// {
//     type Iter = <F as TrIterator<'a>>::Iter;
//
//     fn tr_iter(&'a self, state_id: usize) -> Result<Self::Iter> {
//         self.fst.tr_iter(state_id)
//     }
//
//     unsafe fn tr_iter_unchecked(&'a self, state_id: usize) -> Self::Iter {
//         self.fst.tr_iter_unchecked(state_id)
//     }
// }
//
// impl<'a, F: FstIterator<'a>, T> FstIterator<'a> for FstAddOn<F, T>
// where
//     F::W: 'a,
// {
//     type TrsIter = F::TrsIter;
//     type FstIter = F::FstIter;
//
//     fn fst_iter(&'a self) -> Self::FstIter {
//         self.fst.fst_iter()
//     }
// }
//
// impl<F: Fst, T: Debug> Fst for FstAddOn<F, T>
// where
//     F::W: 'static,
// {
//     fn input_symbols(&self) -> Option<&Arc<SymbolTable>> {
//         self.fst.input_symbols()
//     }
//
//     fn output_symbols(&self) -> Option<&Arc<SymbolTable>> {
//         self.fst.output_symbols()
//     }
//
//     fn set_input_symbols(&mut self, symt: Arc<SymbolTable>) {
//         self.fst.set_input_symbols(symt)
//     }
//
//     fn set_output_symbols(&mut self, symt: Arc<SymbolTable>) {
//         self.fst.set_output_symbols(symt)
//     }
//
//     fn take_input_symbols(&mut self) -> Option<Arc<SymbolTable>> {
//         self.fst.take_input_symbols()
//     }
//
//     fn take_output_symbols(&mut self) -> Option<Arc<SymbolTable>> {
//         self.fst.take_output_symbols()
//     }
// }
//
// impl<F: ExpandedFst, T: Debug + Clone + PartialEq> ExpandedFst for FstAddOn<F, T>
// where
//     F::W: 'static,
// {
//     fn num_states(&self) -> usize {
//         self.fst.num_states()
//     }
// }
//
// impl<F: FstIntoIterator, T: Debug> FstIntoIterator for FstAddOn<F, T>
// where
//     F::W: 'static,
// {
//     type TrsIter = F::TrsIter;
//     type FstIter = F::FstIter;
//
//     fn fst_into_iter(self) -> Self::FstIter {
//         self.fst.fst_into_iter()
//     }
// }
