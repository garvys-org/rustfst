use crate::algorithms::determinize::divisors::CommonDivisor;
use crate::algorithms::determinize::DeterminizeFsaOp;
use crate::algorithms::lazy::{LazyFst, SimpleHashMapCache};
use crate::fst_properties::FstProperties;
use crate::fst_traits::{AllocableFst, CoreFst, Fst, FstIterator, MutableFst, StateIterator};
use crate::semirings::{WeaklyDivisibleSemiring, WeightQuantize};
use crate::{Semiring, StateId, SymbolTable, TrsVec};
use anyhow::Result;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

type InnerLazyFst<W, F, CD, B, BT> =
    LazyFst<W, DeterminizeFsaOp<W, F, CD, B, BT>, SimpleHashMapCache<W>>;

#[derive(Debug)]
pub struct DeterminizeFsa<
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
    F: Fst<W>,
    CD: CommonDivisor<W>,
    B: Borrow<F> + Debug,
    BT: Borrow<[W]> + Debug + PartialEq,
>(InnerLazyFst<W, F, CD, B, BT>, PhantomData<F>);

impl<W, F, CD, B, BT> CoreFst<W> for DeterminizeFsa<W, F, CD, B, BT>
where
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
    F: Fst<W>,
    CD: CommonDivisor<W>,
    B: Borrow<F> + Debug,
    BT: Borrow<[W]> + Debug + PartialEq,
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
        unimplemented!()
    }

    fn num_input_epsilons(&self, state: StateId) -> Result<usize> {
        self.0.num_input_epsilons(state)
    }

    fn num_output_epsilons(&self, state: StateId) -> Result<usize> {
        self.0.num_output_epsilons(state)
    }
}

impl<'a, W, F, CD, B, BT> StateIterator<'a> for DeterminizeFsa<W, F, CD, B, BT>
where
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize + 'a,
    F: Fst<W> + 'a,
    CD: CommonDivisor<W> + 'a,
    B: Borrow<F> + Debug + 'a,
    BT: Borrow<[W]> + Debug + PartialEq + 'a,
{
    type Iter = <InnerLazyFst<W, F, CD, B, BT> as StateIterator<'a>>::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.0.states_iter()
    }
}

impl<'a, W, F, CD, B, BT> FstIterator<'a, W> for DeterminizeFsa<W, F, CD, B, BT>
where
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
    F: Fst<W> + 'a,
    CD: CommonDivisor<W> + 'a,
    B: Borrow<F> + Debug + 'a,
    BT: Borrow<[W]> + Debug + PartialEq + 'a,
{
    type FstIter = <InnerLazyFst<W, F, CD, B, BT> as FstIterator<'a, W>>::FstIter;

    fn fst_iter(&'a self) -> Self::FstIter {
        self.0.fst_iter()
    }
}

impl<W, F, CD, B, BT> Fst<W> for DeterminizeFsa<W, F, CD, B, BT>
where
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
    F: Fst<W> + 'static,
    CD: CommonDivisor<W> + 'static,
    B: Borrow<F> + 'static + std::fmt::Debug,
    BT: Borrow<[W]> + Debug + PartialEq + 'static,
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

impl<W, F, CD, B, BT> DeterminizeFsa<W, F, CD, B, BT>
where
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
    F: Fst<W>,
    CD: CommonDivisor<W>,
    B: Borrow<F> + Debug,
    BT: Borrow<[W]> + PartialEq + Debug,
{
    pub fn new(fst: B, in_dist: Option<BT>, delta: f32) -> Result<Self> {
        let isymt = fst.borrow().input_symbols().cloned();
        let osymt = fst.borrow().output_symbols().cloned();
        let fst_op = DeterminizeFsaOp::new(fst, in_dist, delta)?;
        let fst_cache = SimpleHashMapCache::default();
        let lazy_fst = LazyFst::from_op_and_cache(fst_op, fst_cache, isymt, osymt);
        Ok(DeterminizeFsa(lazy_fst, PhantomData))
    }

    /// Turns the Lazy FST into a static one.
    pub fn compute<F2: MutableFst<W> + AllocableFst<W>>(&self) -> Result<F2> {
        self.0.compute()
    }

    pub fn out_dist(self) -> Result<Vec<W>> {
        self.0.op.out_dist()
    }

    pub fn compute_with_distance<F2: MutableFst<W> + AllocableFst<W>>(
        self,
    ) -> Result<(F2, Vec<W>)> {
        let dfst: F2 = self.compute()?;
        let out_dist = self.out_dist()?;
        Ok((dfst, out_dist))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::algorithms::determinize::DefaultCommonDivisor;
    use crate::fst_impls::VectorFst;
    use crate::semirings::TropicalWeight;

    #[test]
    fn test_determinize_fsa_sync() {
        fn is_sync<T: Sync>() {}
        is_sync::<
            DeterminizeFsa<
                TropicalWeight,
                VectorFst<_>,
                DefaultCommonDivisor,
                Arc<VectorFst<_>>,
                Vec<TropicalWeight>,
            >,
        >();
    }
}
