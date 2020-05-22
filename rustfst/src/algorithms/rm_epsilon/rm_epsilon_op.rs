use std::borrow::Borrow;

use anyhow::Result;

use crate::algorithms::lazy_fst_revamp::FstOp2;
use crate::algorithms::queues::FifoQueue;
use crate::algorithms::rm_epsilon::{RmEpsilonConfig, RmEpsilonState};
use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::TrsVec;
use itertools::Itertools;
use std::cell::RefCell;
use std::sync::Arc;

#[derive(Clone, Eq)]
pub struct RmEpsilonOp<W: Semiring, F: MutableFst<W>, B: Borrow<F>> {
    rmeps_state: RefCell<RmEpsilonState<W, F, B, FifoQueue>>,
}

impl<W: Semiring, F: MutableFst<W>, B: Borrow<F>> std::fmt::Debug for RmEpsilonOp<W, F, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RmEpsilonImpl {{ rmeps_state : {:?}}}",
            self.rmeps_state.borrow()
        )
    }
}

impl<W: Semiring, F: MutableFst<W>, B: Borrow<F>> PartialEq for RmEpsilonOp<W, F, B> {
    fn eq(&self, other: &Self) -> bool {
        self.rmeps_state.eq(&other.rmeps_state)
    }
}

impl<W: Semiring, F: MutableFst<W>, B: Borrow<F>> RmEpsilonOp<W, F, B> {
    pub fn new(fst: B) -> Self {
        Self {
            rmeps_state: RefCell::new(RmEpsilonState::new(
                fst,
                RmEpsilonConfig::new_with_default(FifoQueue::default()),
            )),
        }
    }
}

impl<W: Semiring, F: MutableFst<W>, B: Borrow<F>> FstOp2<W> for RmEpsilonOp<W, F, B> {
    fn compute_start(&self) -> Result<Option<usize>> {
        Ok(self.rmeps_state.borrow().sd_state.fst.borrow().start())
    }

    fn compute_trs_and_final_weight(&self, state: usize) -> Result<(TrsVec<W>, Option<W>)> {
        let (trs, final_weight) = self.rmeps_state.borrow_mut().expand(state)?;
        let zero = W::zero();

        let trs = trs.into_iter().rev().collect_vec();

        let final_weight = if final_weight != zero {
            Some(final_weight)
        } else {
            None
        };

        Ok((TrsVec(Arc::new(trs)), final_weight))
    }
}
