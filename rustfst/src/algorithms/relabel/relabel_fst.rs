use crate::algorithms::lazy::{LazyFst, SimpleHashMapCache};
use crate::fst_properties::FstProperties;
use crate::fst_traits::{AllocableFst, CoreFst, Fst, FstIterator, MutableFst, StateIterator};
use crate::{Semiring, StateId, SymbolTable, TrsVec};
use anyhow::{Result, Context};
use std::borrow::Borrow;
use std::fmt::Debug;
use std::sync::Arc;
use crate::algorithms::relabel::relabel_fst_op::RelabelFstOp;
use crate::algorithms::relabel_pairs::iterator_to_hashmap;

type InnerLazyFst<W, F, B> = LazyFst<W, RelabelFstOp<W, F, B>, SimpleHashMapCache<W>>;

struct RelabelFst<W: Semiring, F: Fst<W>, B: Borrow<F>>(InnerLazyFst<W, F, B>);

impl<W: Semiring, F: Fst<W>, B: Borrow<F>> RelabelFst<W, F, B> {
    pub fn new<I, J>(fst: B, ipairs: I, opairs: J) -> Result<Self>
    where
        I: IntoIterator<Item = (StateId, StateId)>,
        J: IntoIterator<Item = (StateId, StateId)>,
    {
        let map_ilabels = iterator_to_hashmap(ipairs)
            .with_context(|| format_err!("Error while creating the HashMap for ipairs"))?;

        let map_olabels = iterator_to_hashmap(opairs)
            .with_context(|| format_err!("Error while creating the HashMap for opairs"))?;

        let isymt = fst.borrow().input_symbols().cloned();
        let osymt = fst.borrow().output_symbols().cloned();

        let fst_op = RelabelFstOp::new(fst, map_ilabels, map_olabels);
        let fst_cache = SimpleHashMapCache::default();
        Ok(RelabelFst(LazyFst::from_op_and_cache(
            fst_op, fst_cache, isymt, osymt,
        )))
    }

    pub fn compute<F2: MutableFst<W> + AllocableFst<W>>(&self) -> Result<F2> {
        self.0.compute()
    }
}

impl<W, F, B> CoreFst<W> for RelabelFst<W, F, B>
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

impl<'a, W, F, B> StateIterator<'a> for RelabelFst<W, F, B>
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

impl<'a, W, F, B> FstIterator<'a, W> for RelabelFst<W, F, B>
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

impl<W, F, B> Fst<W> for RelabelFst<W, F, B>
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

impl<W, F, B> Debug for RelabelFst<W, F, B>
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
        is_sync::<RelabelFst<TropicalWeight, VectorFst<_>, VectorFst<_>>>();
    }
}
