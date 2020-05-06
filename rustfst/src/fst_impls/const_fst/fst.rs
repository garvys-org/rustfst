use crate::fst_impls::ConstFst;
use crate::fst_traits::{CoreFst, Fst};
use crate::semirings::Semiring;

use crate::{SymbolTable, TrsVec, TrsConst};
use anyhow::{format_err, Result, Error};
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
    type TRS = TrsConst<W>;

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

    fn get_trs(&self, state_id: usize) -> Result<Self::TRS> {
        let state = self
            .states
            .get(state_id)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        Ok(TrsConst{
            trs: Arc::clone(&self.trs),
            pos: state.pos,
            n: state.ntrs
        })
    }

    unsafe fn get_trs_unchecked(&self, state_id: usize) -> Self::TRS {
        let state = self
            .states
            .get_unchecked(state_id);
        TrsConst{
            trs: Arc::clone(&self.trs),
            pos: state.pos,
            n: state.ntrs
        }
    }
}
