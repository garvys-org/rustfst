use std::borrow::Borrow;

use anyhow::Result;

use crate::algorithms::lazy::FstOp2;
use crate::algorithms::queues::FifoQueue;
use crate::algorithms::rm_epsilon::{RmEpsilonInternalConfig, RmEpsilonState};
use crate::fst_properties::mutable_properties::rmepsilon_properties;
use crate::fst_properties::FstProperties;
use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::{StateId, TrsVec};
use itertools::Itertools;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(Clone)]
pub struct RmEpsilonOp<W: Semiring, F: MutableFst<W>, B: Borrow<F>> {
    rmeps_state: RefCell<RmEpsilonState<W, FifoQueue>>,
    properties: FstProperties,
    ghost: PhantomData<F>,
    fst: B,
}

impl<W: Semiring, F: MutableFst<W>, B: Borrow<F>> std::fmt::Debug for RmEpsilonOp<W, F, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RmEpsilonOp {{ rmeps_state : {:?}}}",
            self.rmeps_state.borrow()
        )
    }
}

// impl<W: Semiring, F: MutableFst<W>, B: Borrow<F>> PartialEq for RmEpsilonOp<W, F, B> {
//     fn eq(&self, other: &Self) -> bool {
//         self.rmeps_state.eq(&other.rmeps_state)
//     }
// }

impl<W: Semiring, F: MutableFst<W>, B: Borrow<F>> RmEpsilonOp<W, F, B> {
    pub fn new(fst: B) -> Self {
        let properties = rmepsilon_properties(fst.borrow().properties(), true);
        Self {
            properties,
            rmeps_state: RefCell::new(RmEpsilonState::new(
                fst.borrow().num_states(),
                RmEpsilonInternalConfig::new_with_default(FifoQueue::default()),
            )),
            fst,
            ghost: PhantomData,
        }
    }
}

impl<W: Semiring, F: MutableFst<W>, B: Borrow<F>> FstOp2<W> for RmEpsilonOp<W, F, B> {
    fn compute_start(&self) -> Result<Option<StateId>> {
        Ok(self.fst.borrow().start())
    }

    fn compute_trs_and_final_weight(&self, state: StateId) -> Result<(TrsVec<W>, Option<W>)> {
        let (trs, final_weight) = self
            .rmeps_state
            .borrow_mut()
            .expand::<F, _>(state, self.fst.borrow())?;
        let zero = W::zero();

        let trs = trs.into_iter().rev().collect_vec();

        let final_weight = if final_weight != zero {
            Some(final_weight)
        } else {
            None
        };

        Ok((TrsVec(Arc::new(trs)), final_weight))
    }

    fn properties(&self) -> FstProperties {
        self.properties
    }
}
