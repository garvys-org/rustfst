use std::iter::Enumerate;
use std::iter::Map;
use std::ops::Range;
use std::slice;

use anyhow::Result;

use crate::fst_impls::vector_fst::VectorFstState;
use crate::fst_impls::VectorFst;
use crate::fst_traits::FstIterData;
use crate::fst_traits::{
    FstIntoIterator, FstIterator, FstIteratorMut, MutableTrIterator, StateIterator, TrIterator,
};
use crate::semirings::Semiring;
use crate::StateId;
use crate::Tr;

impl<'a, W: Semiring> StateIterator<'a> for VectorFst<W> {
    type Iter = Range<StateId>;
    fn states_iter(&'a self) -> Self::Iter {
        0..self.states.len()
    }
}

impl<'a, W: 'static + Semiring> TrIterator<'a> for VectorFst<W> {
    type Iter = slice::Iter<'a, Tr<W>>;
    fn tr_iter(&'a self, state_id: StateId) -> Result<Self::Iter> {
        let state = self
            .states
            .get(state_id)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        Ok(state.trs.iter())
    }

    unsafe fn tr_iter_unchecked(&'a self, state_id: usize) -> Self::Iter {
        self.states.get_unchecked(state_id).trs.iter()
    }
}

impl<'a, W: 'static + Semiring> MutableTrIterator<'a> for VectorFst<W> {
    type IterMut = slice::IterMut<'a, Tr<W>>;
    fn tr_iter_mut(&'a mut self, state_id: StateId) -> Result<Self::IterMut> {
        let state = self
            .states
            .get_mut(state_id)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        Ok(state.trs.iter_mut())
    }

    #[inline]
    unsafe fn tr_iter_unchecked_mut(&'a mut self, state_id: usize) -> Self::IterMut {
        self.states.get_unchecked_mut(state_id).trs.iter_mut()
    }
}

impl<W: Semiring> FstIntoIterator for VectorFst<W>
where
    W: 'static,
{
    type TrsIter = std::vec::IntoIter<Tr<W>>;

    // TODO: Change this to impl once the feature has been stabilized
    // #![feature(type_alias_impl_trait)]
    // https://github.com/rust-lang/rust/issues/63063)
    type FstIter = Box<dyn Iterator<Item = FstIterData<W, Self::TrsIter>>>;

    fn fst_into_iter(self) -> Self::FstIter {
        Box::new(
            self.states
                .into_iter()
                .enumerate()
                .map(|(state_id, fst_state)| FstIterData {
                    state_id,
                    num_trs: fst_state.trs.len(),
                    trs: fst_state.trs.into_iter(),
                    final_weight: fst_state.final_weight,
                }),
        )
    }
}

impl<'a, W: Semiring + 'static> FstIterator<'a> for VectorFst<W> {
    type TrsIter = std::slice::Iter<'a, Tr<W>>;
    type FstIter = Map<
        Enumerate<std::slice::Iter<'a, VectorFstState<W>>>,
        Box<dyn FnMut((StateId, &'a VectorFstState<W>)) -> FstIterData<&'a W, Self::TrsIter>>,
    >;
    fn fst_iter(&'a self) -> Self::FstIter {
        self.states
            .iter()
            .enumerate()
            .map(Box::new(|(state_id, fst_state)| FstIterData {
                state_id,
                trs: fst_state.trs.iter(),
                final_weight: fst_state.final_weight.as_ref(),
                num_trs: fst_state.trs.len(),
            }))
    }
}

impl<'a, W: Semiring + 'static> FstIteratorMut<'a> for VectorFst<W> {
    type TrsIter = std::slice::IterMut<'a, Tr<W>>;
    type FstIter = Map<
        Enumerate<std::slice::IterMut<'a, VectorFstState<W>>>,
        Box<
            dyn FnMut(
                (StateId, &'a mut VectorFstState<W>),
            ) -> FstIterData<&'a mut W, Self::TrsIter>,
        >,
    >;

    fn fst_iter_mut(&'a mut self) -> Self::FstIter {
        self.states
            .iter_mut()
            .enumerate()
            .map(Box::new(|(state_id, fst_state)| {
                let n = fst_state.trs.len();
                FstIterData {
                    state_id,
                    trs: fst_state.trs.iter_mut(),
                    final_weight: fst_state.final_weight.as_mut(),
                    num_trs: n,
                }
            }))
    }
}
