use crate::algorithms::compose::lookahead_relabel::lookahead_relabel_fst_op::LookaheadRelabelFstOp;
use crate::algorithms::lazy::{LazyFst, SimpleHashMapCache, iterate_lazy};
use crate::fst_properties::FstProperties;
use crate::fst_traits::{AllocableFst, CoreFst, Fst, FstIterator, MutableFst, StateIterator};
use crate::prelude::compose::LabelReachableData;
use crate::{Semiring, StateId, SymbolTable, TrsVec};
use anyhow::Result;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::sync::Arc;

type InnerLazyFst<W, F, B> = LazyFst<W, LookaheadRelabelFstOp<W, F, B>, SimpleHashMapCache<W>>;

pub struct LookaheadRelabelFst<W: Semiring, F: Fst<W>, B: Borrow<F>>(InnerLazyFst<W, F, B>);

impl<W: Semiring, F: Fst<W>, B: Borrow<F>> LookaheadRelabelFst<W, F, B> {
    pub fn new(
        fst: B,
        label_reachable_data: LabelReachableData,
        relabel_input: bool,
    ) -> Result<Self> {
        let mut isymt = fst.borrow().input_symbols().cloned();
        let mut osymt = fst.borrow().output_symbols().cloned();
        if relabel_input {
            isymt = None;
        } else {
            osymt = None;
        }

        let fst_op = LookaheadRelabelFstOp::new(fst, label_reachable_data, relabel_input);
        let fst_cache = SimpleHashMapCache::default();
        Ok(LookaheadRelabelFst(LazyFst::from_op_and_cache(
            fst_op, fst_cache, isymt, osymt,
        )))
    }

    pub fn compute<F2: MutableFst<W> + AllocableFst<W>>(&self) -> Result<F2> {
        // let underneath_fst = self.0.op.fst.borrow();
        // Trick to compute the underneath lazy fst
        // iterate_lazy(underneath_fst)?;
        self.0.compute()
    }
}

impl<W, F, B> CoreFst<W> for LookaheadRelabelFst<W, F, B>
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

impl<'a, W, F, B> StateIterator<'a> for LookaheadRelabelFst<W, F, B>
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

impl<'a, W, F, B> FstIterator<'a, W> for LookaheadRelabelFst<W, F, B>
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

impl<W, F, B> Fst<W> for LookaheadRelabelFst<W, F, B>
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

impl<W, F, B> Debug for LookaheadRelabelFst<W, F, B>
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
    fn test_lookahead_relabel_fst_sync() {
        fn is_sync<T: Sync>() {}
        is_sync::<LookaheadRelabelFst<TropicalWeight, VectorFst<_>, VectorFst<_>>>();
    }
}
