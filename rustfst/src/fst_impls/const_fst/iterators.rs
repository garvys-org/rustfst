use std::ops::Range;
use std::slice;

use failure::Fallible;
use itertools::Itertools;
use itertools::izip;

use crate::Arc;
use crate::fst_impls::const_fst::data_structure::ConstState;
use crate::fst_impls::ConstFst;
use crate::fst_traits::{ArcIterator, FstIntoIterator, FstIterator, StateIterator};
use crate::semirings::Semiring;
use crate::StateId;

impl<W: Semiring> ConstFst<W> {
    fn state_range(&self) -> Range<usize> {
        0..self.states.len()
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

impl<W: Semiring> FstIntoIterator for ConstFst<W>
where
    W: 'static,
{
    type ArcsIter = std::vec::IntoIter<Arc<W>>;

    // TODO: Change this to impl once the feature has been stabilized
    // #![feature(type_alias_impl_trait)]
    // https://github.com/rust-lang/rust/issues/63063)
    type FstIter = Box<dyn Iterator<Item = (StateId, Self::ArcsIter, Option<W>)>>;

    fn fst_into_iter(mut self) -> Self::FstIter {
        // Here the contiguous arcs are moved into multiple vectors in order to be able to create in iterator for each states.
        // TODO: Find a way to avoid this allocation.
        let mut arcs = Vec::with_capacity(self.states.len());
        for const_state in &self.states {
            arcs.push(self.arcs.drain(0..const_state.narcs).collect_vec())
        }

        Box::new(
            izip!(self.states.into_iter(), arcs.into_iter())
                .enumerate()
                .map(|(state_id, (const_state, arcs_from_state))| {
                    (
                        state_id,
                        arcs_from_state.into_iter(),
                        const_state.final_weight,
                    )
                }),
        )
    }
}

impl<'a, W: Semiring> StateIterator<'a> for ConstFst<W> {
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
        self.0.next().map(StateIndex)
    }
}

impl std::iter::Iterator for ArcIndexIter {
    type Item = ArcIndex;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(ArcIndex)
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

    fn get_state_id(&self, state_idx: Self::StateIndex) -> Fallible<StateId> {
        let _ = self
            .states
            .get(state_idx.0)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_idx.0))?;
        Ok(state_idx.0)
    }

    fn get_arc<'a>(
        &'a self,
        state: Self::StateIndex,
        arc: Self::ArcIndex,
    ) -> Fallible<&'a Arc<Self::W>> {
        let _ = self
            .states
            .get(state.0)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state.0))?;
        Ok(&self.arcs[arc.0])
    }
}

#[cfg(test)]
mod tests {
    use crate::fst_impls::VectorFst;
    use crate::fst_traits::MutableFst;
    use crate::semirings::{ProbabilityWeight, Semiring};

    use super::*;

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

        let const_fst: ConstFst<ProbabilityWeight> = fst.into();

        let states = const_fst
            .states_index_iter()
            .map(|it| const_fst.get_state_id(it))
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

        let const_fst: ConstFst<ProbabilityWeight> = fst.into();

        let mut arcs_ref = vec![];
        for state_index in const_fst.states_index_iter() {
            for arc_index in const_fst.arcs_index_iter(state_index)? {
                arcs_ref.push(const_fst.get_arc(state_index, arc_index)?);
            }
        }

        assert_eq!(
            arcs_ref,
            vec![&arc_1_2, &arc_1_2_bis, &arc_2_3, &arc_2_3_bis]
        );
        Ok(())
    }
}
