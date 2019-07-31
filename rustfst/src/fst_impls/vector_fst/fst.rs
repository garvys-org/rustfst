use failure::Fallible;

use crate::fst_impls::VectorFst;
use crate::fst_traits::{CoreFst, Fst};
use crate::semirings::Semiring;
use crate::StateId;

impl<W: 'static + Semiring> Fst for VectorFst<W> {}

impl<W: 'static + Semiring> CoreFst for VectorFst<W> {
    type W = W;
    fn start(&self) -> Option<StateId> {
        self.start_state
    }

    fn final_weight(&self, state_id: StateId) -> Option<&W> {
        if let Some(state) = self.states.get(state_id) {
            state.final_weight.as_ref()
        } else {
            None
        }
    }

    #[inline]
    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<&Self::W> {
        self.states.get_unchecked(state_id).final_weight.as_ref()
    }

    fn num_arcs(&self, s: StateId) -> Fallible<usize> {
        if let Some(vector_fst_state) = self.states.get(s) {
            Ok(vector_fst_state.num_arcs())
        } else {
            bail!("State {:?} doesn't exist", s);
        }
    }

    #[inline]
    unsafe fn num_arcs_unchecked(&self, s: usize) -> usize {
        self.states.get_unchecked(s).num_arcs()
    }
}
