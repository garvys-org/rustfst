use std::sync::Arc;

use anyhow::Result;

use crate::fst_impls::VectorFst;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{CoreFst, Fst};
use crate::semirings::Semiring;
use crate::{StateId, SymbolTable, Trs, TrsVec};

impl<W: Semiring> Fst<W> for VectorFst<W> {
    fn input_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.isymt.as_ref()
    }

    fn output_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.osymt.as_ref()
    }

    fn set_input_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.isymt = Some(symt)
    }

    fn set_output_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.osymt = Some(symt)
    }

    fn take_input_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.isymt.take()
    }

    fn take_output_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.osymt.take()
    }
}

impl<W: Semiring> CoreFst<W> for VectorFst<W> {
    type TRS = TrsVec<W>;

    fn start(&self) -> Option<StateId> {
        self.start_state
    }

    fn final_weight(&self, state_id: StateId) -> Result<Option<W>> {
        let s = self
            .states
            .get(state_id as usize)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        Ok(s.final_weight.clone())
    }

    #[inline]
    unsafe fn final_weight_unchecked(&self, state_id: StateId) -> Option<W> {
        self.states
            .get_unchecked(state_id as usize)
            .final_weight
            .clone()
    }

    fn num_trs(&self, s: StateId) -> Result<usize> {
        Ok(self
            .states
            .get(s as usize)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", s))?
            .trs
            .len())
    }

    unsafe fn num_trs_unchecked(&self, s: StateId) -> usize {
        self.states.get_unchecked(s as usize).trs.len()
    }

    fn get_trs(&self, state_id: StateId) -> Result<Self::TRS> {
        let state = self
            .states
            .get(state_id as usize)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        // Data is not copied, only Arc
        Ok(state.trs.shallow_clone())
    }

    unsafe fn get_trs_unchecked(&self, state_id: StateId) -> Self::TRS {
        let state = self.states.get_unchecked(state_id as usize);
        // Data is not copied, only Arc
        state.trs.shallow_clone()
    }

    fn properties(&self) -> FstProperties {
        self.properties
    }

    fn num_input_epsilons(&self, state: StateId) -> Result<usize> {
        Ok(self
            .states
            .get(state as usize)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state))?
            .niepsilons)
    }

    fn num_output_epsilons(&self, state: StateId) -> Result<usize> {
        Ok(self
            .states
            .get(state as usize)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state))?
            .noepsilons)
    }
}
