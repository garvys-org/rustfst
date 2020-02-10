use std::iter::Enumerate;
use std::iter::Map;
use std::iter::Skip;
use std::iter::Take;
use std::iter::Zip;
use std::ops::Range;
use std::slice;

use failure::Fallible;
use itertools::{izip, repeat_n, RepeatN};
use itertools::Itertools;

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
        // Here the contiguous arcs are moved into multiple vectors in order to be able to create
        // iterator for each states.
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

//pub fn lol<'a, W>((state_id, (fst_state, arcs)): (StateId, (&'a ConstState<W>, &'a Vec<Arc<W>>))) -> (StateId, impl Iterator<&'a Arc<W>>, Option<&'a W>) {
//    (
//        state_id,
//        arcs.iter().skip(fst_state.pos).take(fst_state.narcs),
//        fst_state.final_weight.as_ref(),
//    )
//}

impl<'a, W: Semiring + 'static> FstIterator<'a> for ConstFst<W> {
    type ArcsIter = Take<Skip<std::slice::Iter<'a, Arc<W>>>>;
    type FstIter = Map<
        Enumerate<Zip<std::slice::Iter<'a, ConstState<W>>, RepeatN<&'a Vec<Arc<W>>>>>,
        Box<
            dyn FnMut(
                (StateId, (&'a ConstState<W>, &'a Vec<Arc<W>>)),
            ) -> (StateId, Self::ArcsIter, Option<&'a W>),
        >,
    >;
    fn fst_iter(&'a self) -> Self::FstIter {
        let it = repeat_n(&self.arcs, self.states.len());
        izip!(self.states.iter(), it).enumerate().map(Box::new(
            |(state_id, (fst_state, arcs)): (StateId, (&'a ConstState<W>, &'a Vec<Arc<W>>))| {
                (
                    state_id,
                    arcs.iter().skip(fst_state.pos).take(fst_state.narcs),
                    fst_state.final_weight.as_ref(),
                )
            },
        ))
    }
}
