use crate::algorithms::determinize::divisors::CommonDivisor;
use crate::algorithms::determinize::DeterminizeFsaOp;
use crate::algorithms::lazy_fst_revamp::{LazyFst, SimpleHashMapCache};
use crate::fst_traits::{CoreFst, Fst, FstIterator, MutableFst, StateIterator};
use crate::semirings::{WeaklyDivisibleSemiring, WeightQuantize};
use crate::{Semiring, SymbolTable, TrsVec};
use anyhow::Result;
use std::sync::Arc;

#[derive(Debug)]
pub struct DeterminizeFsa<
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
    F: Fst<W>,
    CD: CommonDivisor<W>,
>(LazyFst<W, DeterminizeFsaOp<W, F, CD>, SimpleHashMapCache<W>>);

impl<W, F, CD> CoreFst<W> for DeterminizeFsa<W, F, CD>
where
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
    F: Fst<W>,
    CD: CommonDivisor<W>,
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

impl<'a, W, F, CD> StateIterator<'a> for DeterminizeFsa<W, F, CD>
where
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize + 'a,
    F: Fst<W> + 'a,
    CD: CommonDivisor<W> + 'a,
{
    type Iter =
        <LazyFst<W, DeterminizeFsaOp<W, F, CD>, SimpleHashMapCache<W>> as StateIterator<'a>>::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.0.states_iter()
    }
}

impl<'a, W, F, CD> FstIterator<'a, W> for DeterminizeFsa<W, F, CD>
where
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
    F: Fst<W> + 'a,
    CD: CommonDivisor<W> + 'a,
{
    type FstIter = <LazyFst<W, DeterminizeFsaOp<W, F, CD>, SimpleHashMapCache<W>> as FstIterator<
        'a,
        W,
    >>::FstIter;

    fn fst_iter(&'a self) -> Self::FstIter {
        self.0.fst_iter()
    }
}

impl<W, F, CD> Fst<W> for DeterminizeFsa<W, F, CD>
where
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
    F: Fst<W> + 'static,
    CD: CommonDivisor<W> + 'static,
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

impl<W, F, CD> DeterminizeFsa<W, F, CD>
where
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
    F: Fst<W>,
    CD: CommonDivisor<W>,
{
    pub fn new(fst: Arc<F>, in_dist: Option<Arc<Vec<W>>>) -> Result<Self> {
        let isymt = fst.input_symbols().cloned();
        let osymt = fst.output_symbols().cloned();
        let fst_op = DeterminizeFsaOp::new(fst, in_dist)?;
        let fst_cache = SimpleHashMapCache::new();
        let lazy_fst = LazyFst::from_op_and_cache(fst_op, fst_cache, isymt, osymt);
        Ok(DeterminizeFsa(lazy_fst))
    }

    /// Turns the Lazy FST into a static one.
    pub fn compute<F2: MutableFst<W>>(&self) -> Result<F2> {
        self.0.compute()
    }

    pub fn out_dist(self) -> Result<Vec<W>> {
        self.0.op.out_dist()
    }

    pub fn compute_with_distance<F2: MutableFst<W>>(self) -> Result<(F2, Vec<W>)> {
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
        is_sync::<DeterminizeFsa<TropicalWeight, VectorFst<_>, DefaultCommonDivisor>>();
    }
}
