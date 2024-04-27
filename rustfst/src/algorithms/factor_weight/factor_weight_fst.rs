use std::borrow::Borrow;
use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::factor_weight::factor_weight_op::FactorWeightOp;
use crate::algorithms::factor_weight::{FactorIterator, FactorWeightOptions};
use crate::algorithms::lazy::{LazyFst, SimpleHashMapCache};
use crate::fst_properties::FstProperties;
use crate::fst_traits::{AllocableFst, CoreFst, Fst, FstIterator, MutableFst, StateIterator};
use crate::semirings::WeightQuantize;
use crate::{StateId, SymbolTable, TrsVec};

type InnerLazyFst<W, F, B, FI> = LazyFst<W, FactorWeightOp<W, F, B, FI>, SimpleHashMapCache<W>>;

/// The result of weight factoring is a transducer equivalent to the
/// input whose path weights have been factored according to the FactorIterator.
/// States and transitions will be added as necessary. The algorithm is a
/// generalization to arbitrary weights of the second step of the input
/// epsilon-normalization algorithm. This version is a Delayed FST.
pub struct FactorWeightFst<W: WeightQuantize, F: Fst<W>, B: Borrow<F>, FI: FactorIterator<W>>(
    InnerLazyFst<W, F, B, FI>,
);

impl<W, F, B, FI> CoreFst<W> for FactorWeightFst<W, F, B, FI>
where
    W: WeightQuantize,
    F: Fst<W>,
    B: Borrow<F>,
    FI: FactorIterator<W>,
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

impl<'a, W, F, B, FI> StateIterator<'a> for FactorWeightFst<W, F, B, FI>
where
    W: WeightQuantize,
    F: Fst<W> + 'a,
    B: Borrow<F> + 'a,
    FI: FactorIterator<W> + 'a,
{
    type Iter = <InnerLazyFst<W, F, B, FI> as StateIterator<'a>>::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.0.states_iter()
    }
}

impl<'a, W, F, B, FI> FstIterator<'a, W> for FactorWeightFst<W, F, B, FI>
where
    W: WeightQuantize,
    F: Fst<W> + 'a,
    B: Borrow<F> + 'a,
    FI: FactorIterator<W> + 'a,
{
    type FstIter = <InnerLazyFst<W, F, B, FI> as FstIterator<'a, W>>::FstIter;

    fn fst_iter(&'a self) -> Self::FstIter {
        self.0.fst_iter()
    }
}

impl<W, F, B, FI> Fst<W> for FactorWeightFst<W, F, B, FI>
where
    W: WeightQuantize,
    F: Fst<W> + 'static,
    B: Borrow<F> + 'static,
    FI: FactorIterator<W> + 'static,
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

impl<W, F, B, FI> Debug for FactorWeightFst<W, F, B, FI>
where
    W: WeightQuantize,
    F: Fst<W>,
    B: Borrow<F>,
    FI: FactorIterator<W>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<W, F: Fst<W>, B: Borrow<F>, FI: FactorIterator<W>> FactorWeightFst<W, F, B, FI>
where
    W: WeightQuantize,
{
    pub fn new(fst: B, opts: FactorWeightOptions) -> Result<Self> {
        let isymt = fst.borrow().input_symbols().cloned();
        let osymt = fst.borrow().output_symbols().cloned();
        let fst_op = FactorWeightOp::new(fst, opts)?;
        let fst_cache = SimpleHashMapCache::default();
        let lazy_fst = LazyFst::from_op_and_cache(fst_op, fst_cache, isymt, osymt);
        Ok(FactorWeightFst(lazy_fst))
    }

    /// Turns the Lazy FST into a static one.
    pub fn compute<F2: MutableFst<W> + AllocableFst<W>>(&self) -> Result<F2> {
        self.0.compute()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::algorithms::factor_weight::factor_iterators::IdentityFactor;
    use crate::fst_impls::VectorFst;
    use crate::semirings::TropicalWeight;

    #[test]
    fn test_factor_weight_fst_sync() {
        fn is_sync<T: Sync>() {}
        is_sync::<FactorWeightFst<TropicalWeight, VectorFst<_>, VectorFst<_>, IdentityFactor<_>>>();
    }
}
