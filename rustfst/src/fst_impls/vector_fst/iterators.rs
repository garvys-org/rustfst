use std::iter::Enumerate;
use std::iter::Map;
use std::ops::Range;
use std::sync::Arc;

use itertools::Itertools;

use crate::fst_impls::vector_fst::VectorFstState;
use crate::fst_impls::VectorFst;
use crate::fst_traits::FstIterData;
use crate::fst_traits::{FstIntoIterator, FstIterator, StateIterator};
use crate::semirings::Semiring;
use crate::Tr;
use crate::{StateId, Trs};

impl<'a, W: Semiring> StateIterator<'a> for VectorFst<W> {
    type Iter = Range<StateId>;
    fn states_iter(&'a self) -> Self::Iter {
        0..(self.states.len() as StateId)
    }
}

impl<W: Semiring> FstIntoIterator<W> for VectorFst<W> {
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
                        state_id: state_id as StateId,
                        num_trs: trs_vec.len(),
                        trs: trs_vec.into_iter(),
                        final_weight: fst_state.final_weight,
                    }
                }),
        )
    }
}

type States<'a, W> = Enumerate<std::slice::Iter<'a, VectorFstState<W>>>;
type StateToData<'a, W, TRS> =
    Box<dyn FnMut((usize, &'a VectorFstState<W>)) -> FstIterData<W, TRS>>;

impl<'a, W: Semiring + 'static> FstIterator<'a, W> for VectorFst<W> {
    type FstIter = Map<States<'a, W>, StateToData<'a, W, Self::TRS>>;
    fn fst_iter(&'a self) -> Self::FstIter {
        self.states
            .iter()
            .enumerate()
            .map(Box::new(|(state_id, fst_state)| FstIterData {
                state_id: state_id as StateId,
                trs: fst_state.trs.shallow_clone(),
                final_weight: fst_state.final_weight.clone(),
                num_trs: fst_state.trs.len(),
            }))
    }
}
