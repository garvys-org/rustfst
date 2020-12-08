use std::iter::{Enumerate, Map, Zip};
use std::ops::Range;
use std::sync::Arc;

use itertools::Itertools;
use itertools::{izip, repeat_n, RepeatN};

use crate::fst_impls::const_fst::data_structure::ConstState;
use crate::fst_impls::ConstFst;
use crate::fst_traits::FstIterData;
use crate::fst_traits::{FstIntoIterator, FstIterator, StateIterator};
use crate::semirings::Semiring;
use crate::Tr;
use crate::{StateId, TrsConst};

impl<W: Semiring> FstIntoIterator<W> for ConstFst<W>
where
    W: 'static,
{
    type TrsIter = std::vec::IntoIter<Tr<W>>;
    // TODO: Change this to impl once the feature has been stabilized
    // #![feature(type_alias_impl_trait)]
    // https://github.com/rust-lang/rust/issues/63063)
    type FstIter = Box<dyn Iterator<Item = FstIterData<W, Self::TrsIter>>>;

    fn fst_into_iter(mut self) -> Self::FstIter {
        // Here the contiguous trs are moved into multiple vectors in order to be able to create
        // iterator for each states.
        // TODO: Find a way to avoid this allocation.
        let mut v_trs = Vec::with_capacity(self.states.len());
        let trs = Arc::make_mut(&mut self.trs);
        for const_state in &self.states {
            v_trs.push(trs.drain(0..const_state.ntrs).collect_vec())
        }

        Box::new(
            izip!(self.states.into_iter(), v_trs.into_iter())
                .enumerate()
                .map(|(state_id, (const_state, trs_from_state))| FstIterData {
                    state_id: state_id as StateId,
                    trs: trs_from_state.into_iter(),
                    final_weight: const_state.final_weight,
                    num_trs: const_state.ntrs,
                }),
        )
    }
}

impl<'a, W> StateIterator<'a> for ConstFst<W> {
    type Iter = Range<StateId>;
    fn states_iter(&'a self) -> Self::Iter {
        0..(self.states.len() as StateId)
    }
}

type States<'a, W> =
    Enumerate<Zip<std::slice::Iter<'a, ConstState<W>>, RepeatN<&'a Arc<Vec<Tr<W>>>>>>;
type StateToData<'a, W, TRS> =
    Box<dyn FnMut((usize, (&'a ConstState<W>, &'a Arc<Vec<Tr<W>>>))) -> FstIterData<W, TRS>>;

impl<'a, W: Semiring + 'static> FstIterator<'a, W> for ConstFst<W> {
    type FstIter = Map<States<'a, W>, StateToData<'a, W, Self::TRS>>;
    fn fst_iter(&'a self) -> Self::FstIter {
        let it = repeat_n(&self.trs, self.states.len());
        izip!(self.states.iter(), it)
            .enumerate()
            .map(Box::new(|(state_id, (fst_state, trs))| FstIterData {
                state_id: state_id as StateId,
                trs: TrsConst {
                    trs: Arc::clone(trs),
                    pos: fst_state.pos,
                    n: fst_state.ntrs,
                },
                final_weight: fst_state.final_weight.clone(),
                num_trs: fst_state.ntrs,
            }))
    }
}
