use crate::fst_impls::ConstFst;
use crate::fst_traits::{CoreFst, Fst};
use crate::semirings::Semiring;

use crate::SymbolTable;
use anyhow::{format_err, Result};
use std::sync::Arc;

impl<W: Semiring + 'static> Fst for ConstFst<W> {
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
        self.osymt = Some(symt);
    }

    fn take_input_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.isymt.take()
    }

    fn take_output_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.osymt.take()
    }
}

impl<W: Semiring> CoreFst for ConstFst<W> {
    type W = W;

    fn start(&self) -> Option<usize> {
        self.start
    }

    fn final_weight(&self, state_id: usize) -> Result<Option<&Self::W>> {
        let s = self
            .states
            .get(state_id)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        Ok(s.final_weight.as_ref())
    }

    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<&Self::W> {
        self.states.get_unchecked(state_id).final_weight.as_ref()
    }

    fn num_trs(&self, s: usize) -> Result<usize> {
        let const_state = self
            .states
            .get(s)
            .ok_or_else(|| format_err!("State doesn't exist"))?;
        Ok(const_state.ntrs)
    }

    unsafe fn num_trs_unchecked(&self, s: usize) -> usize {
        self.states.get_unchecked(s).ntrs
    }
}
