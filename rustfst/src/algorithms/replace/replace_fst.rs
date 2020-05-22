use std::borrow::Borrow;

use anyhow::Result;

use crate::algorithms::lazy_fst_revamp::{LazyFst, SimpleHashMapCache};
use crate::algorithms::replace::config::ReplaceFstOptions;
use crate::algorithms::replace::replace_fst_op::ReplaceFstOp;
use crate::fst_traits::{CoreFst, Fst, FstIterator, MutableFst, StateIterator};
use crate::semirings::Semiring;
use crate::{Label, SymbolTable, TrsVec};
use std::fmt::Debug;
use std::sync::Arc;

/// ReplaceFst supports lazy replacement of trs in one FST with another FST.
/// This replacement is recursive. ReplaceFst can be used to support a variety of
/// delayed constructions such as recursive transition networks, union, or closure.
pub struct ReplaceFst<W: Semiring, F: Fst<W>, B: Borrow<F>>(
    LazyFst<W, ReplaceFstOp<W, F, B>, SimpleHashMapCache<W>>,
);

impl<W, F, B> ReplaceFst<W, F, B>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F>,
{
    pub fn new(fst_list: Vec<(Label, B)>, root: Label, epsilon_on_replace: bool) -> Result<Self> {
        let mut isymt = None;
        let mut osymt = None;
        if let Some(first_elt) = fst_list.first() {
            isymt = first_elt.1.borrow().input_symbols().cloned();
            osymt = first_elt.1.borrow().output_symbols().cloned();
        }
        let opts = ReplaceFstOptions::new(root, epsilon_on_replace);
        let fst_op = ReplaceFstOp::new(fst_list, opts)?;
        let fst_cache = SimpleHashMapCache::new();
        Ok(ReplaceFst(LazyFst::from_op_and_cache(
            fst_op, fst_cache, isymt, osymt,
        )))
    }

    /// Turns the Lazy FST into a static one.
    pub fn compute<F2: MutableFst<W>>(&self) -> Result<F2> {
        self.0.compute()
    }
}

impl<W, F, B> CoreFst<W> for ReplaceFst<W, F, B>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F>,
{
    type TRS = TrsVec<W>;

    fn start(&self) -> Option<usize> {
        self.0.start()
    }

    fn final_weight(&self, state_id: usize) -> Result<Option<W>> {
        self.0.final_weight(state_id)
    }

    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<W> {
        self.0.final_weight_unchecked(state_id)
    }

    fn get_trs(&self, state_id: usize) -> Result<Self::TRS> {
        self.0.get_trs(state_id)
    }

    unsafe fn get_trs_unchecked(&self, state_id: usize) -> Self::TRS {
        self.0.get_trs_unchecked(state_id)
    }
}

impl<'a, W, F, B> StateIterator<'a> for ReplaceFst<W, F, B>
where
    W: Semiring,
    F: Fst<W> + 'a,
    B: Borrow<F> + 'a,
{
    type Iter =
        <LazyFst<W, ReplaceFstOp<W, F, B>, SimpleHashMapCache<W>> as StateIterator<'a>>::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.0.states_iter()
    }
}

impl<'a, W, F, B> FstIterator<'a, W> for ReplaceFst<W, F, B>
where
    W: Semiring,
    F: Fst<W> + 'a,
    B: Borrow<F> + 'a,
{
    type FstIter =
        <LazyFst<W, ReplaceFstOp<W, F, B>, SimpleHashMapCache<W>> as FstIterator<'a, W>>::FstIter;

    fn fst_iter(&'a self) -> Self::FstIter {
        self.0.fst_iter()
    }
}

impl<W, F, B> Fst<W> for ReplaceFst<W, F, B>
where
    W: Semiring,
    F: Fst<W> + 'static,
    B: Borrow<F> + 'static,
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

impl<W, F, B> Debug for ReplaceFst<W, F, B>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::fst_impls::VectorFst;
    use crate::semirings::TropicalWeight;

    #[test]
    fn test_replace_fst_sync() {
        fn is_sync<T: Sync>() {}
        is_sync::<ReplaceFst<TropicalWeight, VectorFst<_>, VectorFst<_>>>();
    }
}
