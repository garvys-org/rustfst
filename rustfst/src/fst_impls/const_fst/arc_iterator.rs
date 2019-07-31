use std::slice;

use failure::Fallible;

use crate::Arc;
use crate::fst_impls::ConstFst;
use crate::fst_traits::ArcIterator;
use crate::semirings::Semiring;
use crate::StateId;

impl<'a, W: 'static + Semiring> ArcIterator<'a> for ConstFst<W> {
    type Iter = slice::Iter<'a, Arc<W>>;
    fn arcs_iter(&'a self, state_id: StateId) -> Fallible<Self::Iter> {
        let state = self
            .states
            .get(state_id)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        Ok(self.arcs[state.pos..state.pos + state.narcs].iter())
    }

    unsafe fn arcs_iter_unchecked(&'a self, state_id: usize) -> Self::Iter {
        let state = self.states.get_unchecked(state_id);
        self.arcs[state.pos..state.pos + state.narcs].iter()
    }
}