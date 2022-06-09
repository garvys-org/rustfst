use std::borrow::Borrow;
use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::lazy::{LazyFst, SimpleHashMapCache};
use crate::algorithms::replace::config::ReplaceFstOptions;
use crate::algorithms::replace::replace_fst_op::ReplaceFstOp;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{AllocableFst, CoreFst, Fst, FstIterator, MutableFst, StateIterator};
use crate::semirings::Semiring;
use crate::{Label, StateId, SymbolTable, TrsVec};

type InnerLazyFst<W, F, B> = LazyFst<W, ReplaceFstOp<W, F, B>, SimpleHashMapCache<W>>;

/// ReplaceFst supports lazy replacement of trs in one FST with another FST.
/// This replacement is recursive. ReplaceFst can be used to support a variety of
/// delayed constructions such as recursive transition networks, union, or closure.
pub struct ReplaceFst<W: Semiring, F: Fst<W>, B: Borrow<F>>(InnerLazyFst<W, F, B>);

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
        let fst_cache = SimpleHashMapCache::default();
        Ok(ReplaceFst(LazyFst::from_op_and_cache(
            fst_op, fst_cache, isymt, osymt,
        )))
    }

    /// Turns the Lazy FST into a static one.
    pub fn compute<F2: MutableFst<W> + AllocableFst<W>>(&self) -> Result<F2> {
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

    fn start(&self) -> Option<StateId> {
        self.0.start()
    }

    fn final_weight(&self, state_id: StateId) -> Result<Option<W>> {
        self.0.final_weight(state_id)
    }

    unsafe fn final_weight_unchecked(&self, state_id: StateId) -> Option<W> {
        self.0.final_weight_unchecked(state_id)
    }

    fn num_trs(&self, s: StateId) -> Result<usize> {
        self.0.num_trs(s)
    }

    unsafe fn num_trs_unchecked(&self, s: StateId) -> usize {
        self.0.num_trs_unchecked(s)
    }

    fn get_trs(&self, state_id: StateId) -> Result<Self::TRS> {
        self.0.get_trs(state_id)
    }

    unsafe fn get_trs_unchecked(&self, state_id: StateId) -> Self::TRS {
        self.0.get_trs_unchecked(state_id)
    }

    fn properties(&self) -> FstProperties {
        self.0.properties()
    }

    fn num_input_epsilons(&self, state: StateId) -> Result<usize> {
        self.0.num_input_epsilons(state)
    }

    fn num_output_epsilons(&self, state: StateId) -> Result<usize> {
        self.0.num_output_epsilons(state)
    }
}

impl<'a, W, F, B> StateIterator<'a> for ReplaceFst<W, F, B>
where
    W: Semiring,
    F: Fst<W> + 'a,
    B: Borrow<F> + 'a,
{
    type Iter = <InnerLazyFst<W, F, B> as StateIterator<'a>>::Iter;

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
    type FstIter = <InnerLazyFst<W, F, B> as FstIterator<'a, W>>::FstIter;

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
    use crate::fst_impls::VectorFst;
    use crate::semirings::TropicalWeight;

    use super::*;

    #[test]
    fn test_replace_fst_sync() {
        fn is_sync<T: Sync>() {}
        is_sync::<ReplaceFst<TropicalWeight, VectorFst<_>, VectorFst<_>>>();
    }
}
