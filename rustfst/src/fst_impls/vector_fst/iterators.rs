use std::slice;

use crate::fst_impls::VectorFst;
use crate::fst_traits::{
    ArcIterator, FstIterator, FstIteratorMut, MutableArcIterator, StateIterator,
};
use crate::semirings::Semiring;
use crate::Arc;
use crate::StateId;

use failure::Fallible;

use std::ops::Range;

impl<'a, W: Semiring> StateIterator<'a> for VectorFst<W> {
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

    fn get_state_id(&self, state_idx: Self::StateIndex) -> Fallible<StateId> {
        let _ = self
            .states
            .get(state_idx.0)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_idx.0))?;
        Ok(state_idx.0)
    }

    fn get_arc<'a>(
        &'a self,
        state_idx: Self::StateIndex,
        arc: Self::ArcIndex,
    ) -> Fallible<&'a Arc<Self::W>> {
        let state = self
            .states
            .get(state_idx.0)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_idx.0))?;
        state
            .arcs
            .get(arc.0)
            .ok_or_else(|| format_err!("State {:?} | Arcs: {:?} doesn't exit", state_idx.0, arc.0))
    }
}

impl<W: 'static + Semiring> FstIteratorMut for VectorFst<W> {
    fn modify_arc<F>(
        &mut self,
        state_idx: Self::StateIndex,
        arc_idx: Self::ArcIndex,
        modify: F,
    ) -> Fallible<()>
    where
        F: Fn(&mut Arc<Self::W>) -> Fallible<()>,
    {
        let state = self
            .states
            .get_mut(state_idx.0)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_idx.0))?;
        let arc = state.arcs.get_mut(arc_idx.0).ok_or_else(|| {
            format_err!(
                "State {:?} | Arcs: {:?} doesn't exit",
                state_idx.0,
                arc_idx.0
            )
        })?;
        (modify)(arc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::fst_traits::MutableFst;
    use crate::semirings::{ProbabilityWeight, Semiring};

    #[test]
    fn test_states_index_iterator() -> Fallible<()> {
        let mut fst = VectorFst::new();

        // States
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        let s3 = fst.add_state();

        fst.set_start(s1)?;

        // Arcs
        let arc_1_2 = Arc::new(0, 0, ProbabilityWeight::new(1.0), s2);
        let arc_1_2_bis = Arc::new(0, 0, ProbabilityWeight::new(1.0), s2);

        let arc_2_3 = Arc::new(0, 0, ProbabilityWeight::new(1.0), s3);
        let arc_2_3_bis = Arc::new(0, 0, ProbabilityWeight::new(1.0), s3);

        fst.add_arc(s1, arc_1_2.clone())?;
        fst.add_arc(s1, arc_1_2_bis.clone())?;

        fst.add_arc(s2, arc_2_3.clone())?;
        fst.add_arc(s2, arc_2_3_bis.clone())?;

        let states = fst
            .states_index_iter()
            .map(|it| fst.get_state_id(it))
            .collect::<Fallible<Vec<_>>>()?;
        assert_eq!(states, vec![s1, s2, s3]);
        Ok(())
    }

    #[test]
    fn test_arcs_index_iterator() -> Fallible<()> {
        let mut fst = VectorFst::new();

        // States
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        let s3 = fst.add_state();

        fst.set_start(s1)?;

        // Arcs
        let arc_1_2 = Arc::new(0, 0, ProbabilityWeight::new(1.0), s2);
        let arc_1_2_bis = Arc::new(0, 0, ProbabilityWeight::new(1.0), s2);

        let arc_2_3 = Arc::new(0, 0, ProbabilityWeight::new(1.0), s3);
        let arc_2_3_bis = Arc::new(0, 0, ProbabilityWeight::new(1.0), s3);

        fst.add_arc(s1, arc_1_2.clone())?;
        fst.add_arc(s1, arc_1_2_bis.clone())?;

        fst.add_arc(s2, arc_2_3.clone())?;
        fst.add_arc(s2, arc_2_3_bis.clone())?;

        let mut arcs_ref = vec![];
        for state_index in fst.states_index_iter() {
            for arc_index in fst.arcs_index_iter(state_index)? {
                arcs_ref.push(fst.get_arc(state_index, arc_index)?);
            }
        }

        assert_eq!(
            arcs_ref,
            vec![&arc_1_2, &arc_1_2_bis, &arc_2_3, &arc_2_3_bis]
        );
        Ok(())
    }

    #[test]
    fn test_arcs_index_iterator_with_modify() -> Fallible<()> {
        let mut fst = VectorFst::new();

        // States
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        let s3 = fst.add_state();

        fst.set_start(s1)?;

        // Arcs
        let arc_1_2 = Arc::new(0, 0, ProbabilityWeight::new(1.0), s2);
        let arc_1_2_bis = Arc::new(0, 0, ProbabilityWeight::new(1.0), s2);

        let arc_2_3 = Arc::new(0, 0, ProbabilityWeight::new(1.0), s3);
        let arc_2_3_bis = Arc::new(0, 0, ProbabilityWeight::new(1.0), s3);

        fst.add_arc(s1, arc_1_2.clone())?;
        fst.add_arc(s1, arc_1_2_bis.clone())?;

        fst.add_arc(s2, arc_2_3.clone())?;
        fst.add_arc(s2, arc_2_3_bis.clone())?;

        for state_index in fst.states_index_iter() {
            for arc_index in fst.arcs_index_iter(state_index)? {
                fst.modify_arc(state_index, arc_index, |arc| {
                    arc.ilabel = 1;
                    Ok(())
                })?;
            }
        }

        let mut arcs_ref = vec![];
        for state_index in fst.states_index_iter() {
            for arc_index in fst.arcs_index_iter(state_index)? {
                arcs_ref.push(fst.get_arc(state_index, arc_index)?.ilabel);
            }
        }

        assert_eq!(arcs_ref, vec![1; 4]);
        Ok(())
    }
}
