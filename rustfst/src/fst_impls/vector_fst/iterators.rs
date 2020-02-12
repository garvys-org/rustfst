use std::iter::Enumerate;
use std::iter::Map;
use std::ops::Range;
use std::slice;

use failure::Fallible;

use crate::fst_impls::vector_fst::VectorFstState;
use crate::fst_impls::VectorFst;
use crate::fst_traits::FstIterData;
use crate::fst_traits::{
    ArcIterator, FstIntoIterator, FstIterator, FstIteratorMut, MutableArcIterator, StateIterator,
};
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
    type FstIter = Box<dyn Iterator<Item = FstIterData<W, Self::ArcsIter>>>;

    fn fst_into_iter(self) -> Self::FstIter {
        Box::new(
            self.states
                .into_iter()
                .enumerate()
                .map(|(state_id, fst_state)| FstIterData {
                    state_id,
                    num_arcs: fst_state.arcs.len(),
                    arcs: fst_state.arcs.into_iter(),
                    final_weight: fst_state.final_weight,
                }),
        )
    }
}

impl<'a, W: Semiring + 'static> FstIterator<'a> for VectorFst<W> {
    type ArcsIter = std::slice::Iter<'a, Arc<W>>;
    type FstIter = Map<
        Enumerate<std::slice::Iter<'a, VectorFstState<W>>>,
        Box<dyn FnMut((StateId, &'a VectorFstState<W>)) -> FstIterData<&'a W, Self::ArcsIter>>,
    >;
    fn fst_iter(&'a self) -> Self::FstIter {
        self.states
            .iter()
            .enumerate()
            .map(Box::new(|(state_id, fst_state)| FstIterData {
                state_id,
                arcs: fst_state.arcs.iter(),
                final_weight: fst_state.final_weight.as_ref(),
                num_arcs: fst_state.arcs.len(),
            }))
    }
}

impl<'a, W: Semiring + 'static> FstIteratorMut<'a> for VectorFst<W> {
    type ArcsIter = std::slice::IterMut<'a, Arc<W>>;
    type FstIter = Map<
        Enumerate<std::slice::IterMut<'a, VectorFstState<W>>>,
        Box<
            dyn FnMut(
                (StateId, &'a mut VectorFstState<W>),
            ) -> (StateId, Self::ArcsIter, Option<&'a mut W>),
        >,
    >;

    fn fst_iter_mut(&'a mut self) -> Self::FstIter {
        self.states
            .iter_mut()
            .enumerate()
            .map(Box::new(|(state_id, fst_state)| {
                (
                    state_id,
                    fst_state.arcs.iter_mut(),
                    fst_state.final_weight.as_mut(),
                )
            }))
    }
}
