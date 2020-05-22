use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::replace::ReplaceFst;
use crate::fst_traits::{AllocableFst, CoreFst, Fst, FstIterator, MutableFst, StateIterator};
use crate::semirings::Semiring;
use crate::{SymbolTable, Tr, TrsVec, EPS_LABEL};

/// Computes the concatenation (product) of two FSTs; this version is a delayed
/// FST. If FST1 transduces string x to y with weight a and FST2 transduces
/// string w to v with weight b, then their concatenation transduces string xw
/// to yv with Times(a, b).
#[derive(Debug)]
pub struct ConcatFst<W: Semiring, F: Fst<W> + 'static>(ReplaceFst<W, F, F>);

impl<W, F> ConcatFst<W, F>
where
    W: Semiring,
    F: MutableFst<W> + AllocableFst<W>,
{
    //TODO: Use a borrow and not a move
    //TODO: Allow fsts of different types
    pub fn new(fst1: F, fst2: F) -> Result<Self> {
        let mut rfst = F::new();
        rfst.add_states(3);
        unsafe { rfst.set_start_unchecked(0) };
        unsafe { rfst.set_final_unchecked(2, W::one()) };
        if let Some(isymt) = fst1.input_symbols() {
            rfst.set_input_symbols(Arc::clone(isymt));
        }
        if let Some(osymt) = fst1.output_symbols() {
            rfst.set_output_symbols(Arc::clone(osymt));
        }
        unsafe { rfst.add_tr_unchecked(0, Tr::new(EPS_LABEL, std::usize::MAX, W::one(), 1)) };
        unsafe { rfst.add_tr_unchecked(1, Tr::new(EPS_LABEL, std::usize::MAX - 1, W::one(), 2)) };

        let mut fst_tuples = Vec::with_capacity(3);
        fst_tuples.push((0, rfst));
        fst_tuples.push((std::usize::MAX, fst1));
        fst_tuples.push((std::usize::MAX - 1, fst2));

        Ok(ConcatFst(ReplaceFst::new(fst_tuples, 0, false)?))
    }

    /// Turns the Lazy FST into a static one.
    pub fn compute<F2: MutableFst<W>>(&self) -> Result<F2> {
        self.0.compute()
    }
}

impl<W, F> CoreFst<W> for ConcatFst<W, F>
where
    W: Semiring,
    F: Fst<W>,
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

    fn num_trs(&self, s: usize) -> Result<usize> {
        self.0.num_trs(s)
    }

    unsafe fn num_trs_unchecked(&self, s: usize) -> usize {
        self.0.num_trs_unchecked(s)
    }

    fn get_trs(&self, state_id: usize) -> Result<Self::TRS> {
        self.0.get_trs(state_id)
    }

    unsafe fn get_trs_unchecked(&self, state_id: usize) -> Self::TRS {
        self.0.get_trs_unchecked(state_id)
    }
}

impl<'a, W, F> StateIterator<'a> for ConcatFst<W, F>
where
    W: Semiring,
    F: Fst<W> + 'a,
{
    type Iter = <ReplaceFst<W, F, F> as StateIterator<'a>>::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.0.states_iter()
    }
}

impl<W, F> Fst<W> for ConcatFst<W, F>
where
    W: Semiring,
    F: Fst<W> + 'static,
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

impl<'a, W, F> FstIterator<'a, W> for ConcatFst<W, F>
where
    W: Semiring,
    F: Fst<W> + 'a,
{
    type FstIter = <ReplaceFst<W, F, F> as FstIterator<'a, W>>::FstIter;

    fn fst_iter(&'a self) -> Self::FstIter {
        self.0.fst_iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::fst_impls::VectorFst;
    use crate::semirings::TropicalWeight;

    #[test]
    fn test_concat_fst_sync() {
        fn is_sync<T: Sync>() {}
        is_sync::<ConcatFst<TropicalWeight, VectorFst<_>>>();
    }
}
