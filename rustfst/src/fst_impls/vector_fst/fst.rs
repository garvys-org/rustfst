use std::sync::Arc;

use anyhow::{Error, Result};

use crate::fst_impls::VectorFst;
use crate::fst_traits::{CoreFst, Fst};
use crate::semirings::Semiring;
use crate::{StateId, SymbolTable, Trs, TrsVec};

impl<W: 'static + Semiring> Fst<W> for VectorFst<W> {
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

impl<W: 'static + Semiring> CoreFst<W> for VectorFst<W> {
    type TRS = TrsVec<W>;

    fn start(&self) -> Option<StateId> {
        self.start_state
    }

    fn final_weight(&self, state_id: StateId) -> Result<Option<W>> {
        let s = self
            .states
            .get(state_id)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        Ok(s.final_weight.clone())
    }

    #[inline]
    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<W> {
        self.states.get_unchecked(state_id).final_weight.clone()
    }

    fn num_trs(&self, s: StateId) -> Result<usize> {
        if let Some(vector_fst_state) = self.states.get(s) {
            Ok(vector_fst_state.num_trs())
        } else {
            bail!("State {:?} doesn't exist", s);
        }
    }

    #[inline]
    unsafe fn num_trs_unchecked(&self, s: usize) -> usize {
        self.states.get_unchecked(s).num_trs()
    }

    fn get_trs(&self, state_id: usize) -> Result<Self::TRS> {
        let state = self
            .states
            .get(state_id)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        // Data is not copied, only Arc
        Ok(state.trs.shallow_clone())
    }

    unsafe fn get_trs_unchecked(&self, state_id: usize) -> Self::TRS {
        let state = self.states.get_unchecked(state_id);
        // Data is not copied, only Arc
        state.trs.shallow_clone()
    }
}
