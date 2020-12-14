use std::ops::Deref;
use std::sync::Arc;

use anyhow::Result;

use crate::fst_properties::FstProperties;
use crate::fst_traits::{CoreFst, ExpandedFst, Fst, FstIntoIterator, FstIterator, StateIterator};
use crate::semirings::Semiring;
use crate::{StateId, SymbolTable};

impl<W: Semiring, F: Fst<W>> Fst<W> for Arc<F> {
    fn input_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.deref().input_symbols()
    }

    fn output_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.deref().output_symbols()
    }

    fn set_input_symbols(&mut self, _symt: Arc<SymbolTable>) {
        unimplemented!()
    }

    fn set_output_symbols(&mut self, _symt: Arc<SymbolTable>) {
        unimplemented!()
    }

    fn take_input_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        unimplemented!()
    }

    fn take_output_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        unimplemented!()
    }
}

impl<W: Semiring, F: ExpandedFst<W>> ExpandedFst<W> for Arc<F> {
    fn num_states(&self) -> usize {
        self.deref().num_states()
    }
}

impl<W: Semiring, F: CoreFst<W>> CoreFst<W> for Arc<F> {
    type TRS = F::TRS;

    fn start(&self) -> Option<StateId> {
        self.deref().start()
    }

    fn final_weight(&self, state_id: StateId) -> Result<Option<W>> {
        self.deref().final_weight(state_id)
    }

    unsafe fn final_weight_unchecked(&self, state_id: StateId) -> Option<W> {
        self.deref().final_weight_unchecked(state_id)
    }

    fn num_trs(&self, s: StateId) -> Result<usize> {
        self.deref().num_trs(s)
    }

    unsafe fn num_trs_unchecked(&self, s: StateId) -> usize {
        self.deref().num_trs_unchecked(s)
    }

    fn get_trs(&self, state_id: StateId) -> Result<Self::TRS> {
        self.deref().get_trs(state_id)
    }

    unsafe fn get_trs_unchecked(&self, state_id: StateId) -> Self::TRS {
        self.deref().get_trs_unchecked(state_id)
    }

    fn properties(&self) -> FstProperties {
        self.deref().properties()
    }

    fn num_input_epsilons(&self, state: StateId) -> Result<usize> {
        self.deref().num_input_epsilons(state)
    }

    fn num_output_epsilons(&self, state: StateId) -> Result<usize> {
        self.deref().num_output_epsilons(state)
    }
}

impl<'a, W: Semiring + 'a, F: FstIterator<'a, W>> FstIterator<'a, W> for Arc<F> {
    type FstIter = F::FstIter;

    fn fst_iter(&'a self) -> Self::FstIter {
        self.deref().fst_iter()
    }
}

impl<'a, F: StateIterator<'a>> StateIterator<'a> for Arc<F> {
    type Iter = F::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.deref().states_iter()
    }
}

impl<W: Semiring, F: FstIntoIterator<W>> FstIntoIterator<W> for Arc<F> {
    type TrsIter = F::TrsIter;
    type FstIter = F::FstIter;

    fn fst_into_iter(self) -> Self::FstIter {
        unimplemented!()
    }
}
