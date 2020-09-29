use std::fmt::Debug;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::Result;

use crate::fst_properties::FstProperties;
use crate::{Semiring, StateId, TrsVec};

pub trait FstOp2<W: Semiring>: Debug {
    // was FstImpl
    fn compute_start(&self) -> Result<Option<StateId>>;
    fn compute_trs_and_final_weight(&self, id: usize) -> Result<(TrsVec<W>, Option<W>)>;

    fn properties(&self) -> FstProperties;
}

impl<W: Semiring, F: FstOp2<W>> FstOp2<W> for Arc<F> {
    fn compute_start(&self) -> Result<Option<StateId>> {
        self.deref().compute_start()
    }

    fn compute_trs_and_final_weight(&self, id: usize) -> Result<(TrsVec<W>, Option<W>)> {
        self.deref().compute_trs_and_final_weight(id)
    }

    // Computed at construction time
    fn properties(&self) -> FstProperties {
        self.deref().properties()
    }
}

impl<W: Semiring, F: FstOp2<W>> FstOp2<W> for Rc<F> {
    fn compute_start(&self) -> Result<Option<StateId>> {
        self.deref().compute_start()
    }

    fn compute_trs_and_final_weight(&self, id: usize) -> Result<(TrsVec<W>, Option<W>)> {
        self.deref().compute_trs_and_final_weight(id)
    }

    // Computed at construction time
    fn properties(&self) -> FstProperties {
        self.deref().properties()
    }
}
