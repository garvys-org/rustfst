use crate::fst_impls::ConstFst;
use crate::fst_traits::{CoreFst, Fst};
use crate::semirings::Semiring;

use failure::{format_err, Fallible};

impl<W: Semiring + 'static> Fst for ConstFst<W> {}

impl<W: Semiring> CoreFst for ConstFst<W> {
    type W = W;

    fn start(&self) -> Option<usize> {
        self.start
    }

    fn final_weight(&self, state_id: usize) -> Option<&Self::W> {
        if let Some(state) = self.states.get(state_id) {
            state.final_weight.as_ref()
        } else {
            None
        }
    }

    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<&Self::W> {
        self.states.get_unchecked(state_id).final_weight.as_ref()
    }

    fn num_arcs(&self, s: usize) -> Fallible<usize> {
        let const_state = self
            .states
            .get(s)
            .ok_or_else(|| format_err!("State doesn't exist"))?;
        Ok(const_state.narcs)
    }

    unsafe fn num_arcs_unchecked(&self, s: usize) -> usize {
        self.states.get_unchecked(s).narcs
    }
}
