use crate::fst_impls::const_fst::data_structure::ConstState;
use std::slice;

use failure::Fallible;
use crate::fst_impls::ConstFst;
use crate::fst_traits::{ StateIterator, ArcIterator, FstIterator };
use crate::Arc;
use crate::StateId;
use std::ops::Range;
use crate::semirings::Semiring;

impl<W: Semiring> ConstFst<W> {
    fn state_range(&self) -> Range<usize> {
        (0..self.states.len())
    }

    fn arc_range(&self, state: &ConstState<W>) -> Range<usize> {
        state.pos..state.pos + state.narcs
    }
}

impl<'a, W: 'static + Semiring> ArcIterator<'a> for ConstFst<W> {
    type Iter = slice::Iter<'a, Arc<W>>;
    fn arcs_iter(&'a self, state_id: StateId) -> Fallible<Self::Iter> {
        let state = self
            .states
            .get(state_id)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        Ok(self.arcs[self.arc_range(state)].iter())
    }

    unsafe fn arcs_iter_unchecked(&'a self, state_id: usize) -> Self::Iter {
        let state = self.states.get_unchecked(state_id);
        self.arcs[self.arc_range(state)].iter()
    }
}

impl<'a, W: 'a + Semiring> StateIterator<'a> for ConstFst<W> {
    type Iter = Range<StateId>;
    fn states_iter(&'a self) -> Self::Iter {
        self.state_range()
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

impl<W: Semiring> FstIterator for ConstFst<W> {
    type StateIndex = StateIndex;
    type ArcIndex = ArcIndex;
    type ArcIter = ArcIndexIter;
    type StateIter = StateIndexIter;

    fn states_index_iter(&self) -> Self::StateIter {
        StateIndexIter(self.state_range())
    }

    fn arcs_index_iter(&self, state: Self::StateIndex) -> Fallible<Self::ArcIter> {

        let state = self
            .states
            .get(state.0)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state.0))?;
        Ok(ArcIndexIter(self.arc_range(state)))
    }

    fn get_arc<'a>(&'a self, state: Self::StateIndex, arc: Self::ArcIndex) -> Fallible<&'a Arc<Self::W>> {
        let _ = self
            .states
            .get(state.0)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state.0))?;
        Ok(&self.arcs[arc.0])
    }
}