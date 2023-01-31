use std::borrow::Borrow;

use anyhow::Result;

use crate::algorithms::lazy::FstOp2;
use crate::algorithms::queues::FifoQueue;
use crate::algorithms::rm_epsilon::{RmEpsilonInternalConfig, RmEpsilonState};
use crate::fst_properties::mutable_properties::rmepsilon_properties;
use crate::fst_properties::FstProperties;
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{StateId, TrsVec};
use itertools::Itertools;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

pub struct RmEpsilonOp<W: Semiring, F: Fst<W>, B: Borrow<F>> {
    rmeps_state: Mutex<RmEpsilonState<W, FifoQueue, F, B>>,
    properties: FstProperties,
}

impl<W: Semiring, F: Fst<W>, B: Borrow<F>> std::fmt::Debug for RmEpsilonOp<W, F, B> {
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

impl<W: Semiring, F: Fst<W>, B: Borrow<F>> RmEpsilonOp<W, F, B> {
    pub fn new(fst: B) -> Self {
        let properties = rmepsilon_properties(fst.borrow().properties(), true);
        Self {
            properties,
            rmeps_state: Mutex::new(RmEpsilonState::new(
                fst,
                RmEpsilonInternalConfig::new_with_default(FifoQueue::default()),
            )),
        }
    }
}

impl<W: Semiring, F: Fst<W>, B: Borrow<F>> FstOp2<W> for RmEpsilonOp<W, F, B> {
    fn compute_start(&self) -> Result<Option<StateId>> {
        let mutex = self.rmeps_state.lock().unwrap();
        let rm_state = mutex.deref().borrow();
        Ok(rm_state.fst().borrow().start())
    }

    fn compute_trs_and_final_weight(&self, state: StateId) -> Result<(TrsVec<W>, Option<W>)> {
        let mut mutex = self.rmeps_state.lock().unwrap();
        let rm_state = mutex.deref_mut();
        let (trs, final_weight) = rm_state.expand(state)?;
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
