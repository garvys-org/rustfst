use std::rc::Rc;

use anyhow::Result;

use crate::fst_impls::VectorFst;
use crate::fst_traits::{CoreFst, Fst};
use crate::semirings::Semiring;
use crate::{StateId, SymbolTable};

impl<W: 'static + Semiring> Fst for VectorFst<W> {
    fn input_symbols(&self) -> Option<Rc<SymbolTable>> {
        // Rc is incremented, SymbolTable is not duplicated
        self.isymt.clone()
    }

    fn output_symbols(&self) -> Option<Rc<SymbolTable>> {
        self.osymt.clone()
    }

    fn set_input_symbols(&mut self, symt: Rc<SymbolTable>) {
        self.isymt = Some(Rc::clone(&symt))
    }

    fn set_output_symbols(&mut self, symt: Rc<SymbolTable>) {
        self.osymt = Some(Rc::clone(&symt));
    }

    fn unset_input_symbols(&mut self) -> Option<Rc<SymbolTable>> {
        self.isymt.take()
    }

    fn unset_output_symbols(&mut self) -> Option<Rc<SymbolTable>> {
        self.osymt.take()
    }
}

impl<W: 'static + Semiring> CoreFst for VectorFst<W> {
    type W = W;
    fn start(&self) -> Option<StateId> {
        self.start_state
    }

    fn final_weight(&self, state_id: StateId) -> Result<Option<&W>> {
        let s = self
            .states
            .get(state_id)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        Ok(s.final_weight.as_ref())
    }

    #[inline]
    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<&Self::W> {
        self.states.get_unchecked(state_id).final_weight.as_ref()
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
}
