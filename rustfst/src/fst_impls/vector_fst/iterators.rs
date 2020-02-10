use std::ops::Range;
use std::slice;

use failure::Fallible;

use crate::fst_impls::VectorFst;
use crate::fst_traits::{ArcIterator, FstIntoIterator, MutableArcIterator, StateIterator};
use crate::semirings::Semiring;
use crate::Arc;
use crate::StateId;

impl<'a, W: Semiring> StateIterator<'a> for VectorFst<W> {
    type Iter = Range<StateId>;
    fn states_iter(&'a self) -> Self::Iter {
        0..self.states.len()
    }
}

impl<'a, W: 'static + Semiring> ArcIterator<'a> for VectorFst<W> {
    type Iter = slice::Iter<'a, Arc<W>>;
    fn arcs_iter(&'a self, state_id: StateId) -> Fallible<Self::Iter> {
        let state = self
            .states
            .get(state_id)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        Ok(state.arcs.iter())
    }

    unsafe fn arcs_iter_unchecked(&'a self, state_id: usize) -> Self::Iter {
        self.states.get_unchecked(state_id).arcs.iter()
    }
}

impl<'a, W: 'static + Semiring> MutableArcIterator<'a> for VectorFst<W> {
    type IterMut = slice::IterMut<'a, Arc<W>>;
    fn arcs_iter_mut(&'a mut self, state_id: StateId) -> Fallible<Self::IterMut> {
        let state = self
            .states
            .get_mut(state_id)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        Ok(state.arcs.iter_mut())
    }

    #[inline]
    unsafe fn arcs_iter_unchecked_mut(&'a mut self, state_id: usize) -> Self::IterMut {
        self.states.get_unchecked_mut(state_id).arcs.iter_mut()
    }
}

impl<W: Semiring> FstIntoIterator for VectorFst<W>
where
    W: 'static,
{
    type ArcsIter = std::vec::IntoIter<Arc<W>>;

    // TODO: Change this to impl once the feature has been stabilized
    // #![feature(type_alias_impl_trait)]
    // https://github.com/rust-lang/rust/issues/63063)
    type FstIter = Box<dyn Iterator<Item = (StateId, Self::ArcsIter, Option<W>)>>;

    fn fst_into_iter(self) -> Self::FstIter {
        Box::new(
            self.states
                .into_iter()
                .enumerate()
                .map(|(state_id, fst_state)| {
                    (state_id, fst_state.arcs.into_iter(), fst_state.final_weight)
                }),
        )
    }
}
