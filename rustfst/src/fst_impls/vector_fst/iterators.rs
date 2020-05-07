use std::iter::Enumerate;
use std::iter::Map;
use std::ops::Range;
use std::slice;

use anyhow::Result;

use crate::fst_impls::vector_fst::VectorFstState;
use crate::fst_impls::VectorFst;
use crate::fst_traits::FstIterData;
use crate::fst_traits::{
    FstIntoIterator, FstIterator, FstIteratorMut, StateIterator
};
use crate::semirings::Semiring;
use crate::{StateId, Trs};
use crate::Tr;
use std::sync::Arc;

impl<'a, W: Semiring> StateIterator<'a> for VectorFst<W> {
    type Iter = Range<StateId>;
    fn states_iter(&'a self) -> Self::Iter {
        0..self.states.len()
    }
}

// impl<W: Semiring> FstIntoIterator for VectorFst<W>
// where
//     W: 'static,
// {
//     // TODO: Change this to impl once the feature has been stabilized
//     // #![feature(type_alias_impl_trait)]
//     // https://github.com/rust-lang/rust/issues/63063)
//     type FstIter = Box<dyn Iterator<Item = FstIterData<W, Self::TRS>>>;
//
//     fn fst_into_iter(self) -> Self::FstIter {
//         Box::new(
//             self.states
//                 .into_iter()
//                 .enumerate()
//                 .map(|(state_id, fst_state)| FstIterData {
//                     state_id,
//                     num_trs: fst_state.trs.len(),
//                     trs: fst_state.trs.shallow_clone(),
//                     final_weight: fst_state.final_weight,
//                 }),
//         )
//     }
// }

impl<'a, W: Semiring + 'static> FstIterator<'a> for VectorFst<W> {
    type FstIter = Map<
        Enumerate<std::slice::Iter<'a, VectorFstState<W>>>,
        Box<dyn FnMut((StateId, &'a VectorFstState<W>)) -> FstIterData<&'a W, Self::TRS>>,
    >;
    fn fst_iter(&'a self) -> Self::FstIter {
        self.states
            .iter()
            .enumerate()
            .map(Box::new(|(state_id, fst_state)| FstIterData {
                state_id,
                trs: fst_state.trs.shallow_clone(),
                final_weight: fst_state.final_weight.as_ref(),
                num_trs: fst_state.trs.len(),
            }))
    }
}

impl<'a, W: Semiring + 'static> FstIteratorMut<'a> for VectorFst<W> {
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
