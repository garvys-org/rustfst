use std::slice;

use crate::fst_impls::VectorFst;
use crate::fst_traits::{ArcIterator, MutableArcIterator, StateIterator, FstIterator, FstIteratorMut};
use crate::semirings::Semiring;
use crate::Arc;
use crate::StateId;

use failure::Fallible;

use std::ops::Range;


impl<'a, W: 'a + Semiring> StateIterator<'a> for VectorFst<W> {
    type Iter = Range<StateId>;
    fn states_iter(&'a self) -> Self::Iter {
        (0..self.states.len())
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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct StateIndex(StateId);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ArcIndex(usize);

#[derive(Debug, Clone, PartialEq)]
pub struct StateIndexIter(Range<usize>);
#[derive(Debug, Clone, PartialEq)]
pub struct ArcIndexIter(Range<usize>);

impl std::iter::Iterator for StateIndexIter {
    type Item = StateIndex;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|it| StateIndex(it))
    }
}

impl std::iter::Iterator for ArcIndexIter {
    type Item = ArcIndex;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|it| ArcIndex(it))
    }
}

impl<W: 'static + Semiring> FstIterator for VectorFst<W> {
    type StateIndex = StateIndex;
    type ArcIndex = ArcIndex;
    type ArcIter = ArcIndexIter;
    type StateIter = StateIndexIter;

    fn states_index_iter(&self) -> Self::StateIter {
        StateIndexIter(self.states_iter())
    }

    fn arcs_index_iter(&self, state: Self::StateIndex) -> Fallible<Self::ArcIter> {
        let state = self
            .states
            .get(state.0)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state.0))?;
        Ok(ArcIndexIter(0..state.num_arcs()))
    }

    fn get_arc<'a>(&'a self, state_idx: Self::StateIndex, arc: Self::ArcIndex) -> Fallible<&'a Arc<Self::W>> {
        let state = self
            .states
            .get(state_idx.0)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_idx.0))?;
        state.arcs.get(arc.0).ok_or_else(|| format_err!("State {:?} | Arcs: {:?} doesn't exit", state_idx.0, arc.0))
    }
}

impl<W: 'static + Semiring> FstIteratorMut for VectorFst<W> {
    fn modify_arc<F>(&mut self, state_idx: Self::StateIndex, arc_idx: Self::ArcIndex, modify: F) -> Fallible<()> where F: Fn(&mut Arc<Self::W>) -> Fallible<()> {
        let state = self
            .states
            .get_mut(state_idx.0)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_idx.0))?;
        let arc = state.arcs.get_mut(arc_idx.0).ok_or_else(|| format_err!("State {:?} | Arcs: {:?} doesn't exit", state_idx.0, arc_idx.0))?;
        (modify)(arc)
    }
}
