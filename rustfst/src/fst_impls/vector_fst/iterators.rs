use std::iter::Enumerate;
use std::iter::Map;
use std::ops::Range;
use std::slice;
use std::sync::Arc;

use itertools::Itertools;

use crate::fst_impls::vector_fst::VectorFstState;
use crate::fst_impls::VectorFst;
use crate::fst_traits::FstIterData;
use crate::fst_traits::{FstIntoIterator, FstIterator, FstIteratorMut, StateIterator};
use crate::semirings::Semiring;
use crate::Tr;
use crate::{StateId, Trs};

impl<'a, W: Semiring> StateIterator<'a> for VectorFst<W> {
    type Iter = Range<StateId>;
    fn states_iter(&'a self) -> Self::Iter {
        0..self.states.len()
    }
}

impl<W: Semiring> FstIntoIterator<W> for VectorFst<W>
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
                .map(|(state_id, fst_state)| {
                    let mut trs = fst_state.trs.0;
                    let trs_vec = Arc::make_mut(&mut trs).drain(..).collect_vec();
                    FstIterData {
                        state_id,
                        num_trs: trs_vec.len(),
                        trs: trs_vec.into_iter(),
                        final_weight: fst_state.final_weight,
                    }
                }),
        )
    }
}

impl<'a, W: Semiring + 'static> FstIterator<'a, W> for VectorFst<W> {
    type FstIter = Map<
        Enumerate<std::slice::Iter<'a, VectorFstState<W>>>,
        Box<dyn FnMut((StateId, &'a VectorFstState<W>)) -> FstIterData<W, Self::TRS>>,
    >;
    fn fst_iter(&'a self) -> Self::FstIter {
        self.states
            .iter()
            .enumerate()
            .map(Box::new(|(state_id, fst_state)| FstIterData {
                state_id,
                trs: fst_state.trs.shallow_clone(),
                final_weight: fst_state.final_weight.clone(),
                num_trs: fst_state.trs.len(),
            }))
    }
}

impl<'a, W: Semiring + 'static> FstIteratorMut<'a, W> for VectorFst<W> {
    type FstIter = Map<
        Enumerate<std::slice::IterMut<'a, VectorFstState<W>>>,
        Box<
            dyn FnMut(
                (StateId, &'a mut VectorFstState<W>),
            ) -> FstIterData<&'a mut W, slice::IterMut<'a, Tr<W>>>,
        >,
    >;

    fn fst_iter_mut(&'a mut self) -> Self::FstIter {
        self.states
            .iter_mut()
            .enumerate()
            .map(Box::new(|(state_id, fst_state)| {
                let n = fst_state.trs.len();
                let trs = Arc::make_mut(&mut fst_state.trs.0);
                FstIterData {
                    state_id,
                    trs: trs.iter_mut(),
                    final_weight: fst_state.final_weight.as_mut(),
                    num_trs: n,
                }
            }))
    }
}
