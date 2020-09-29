use std::fmt::Debug;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::Result;

use crate::fst_properties::FstProperties;
use crate::semirings::Semiring;
use crate::{StateId, TrsVec};

pub trait FstOp<W: Semiring>: Debug {
    // was FstImpl
    fn compute_start(&self) -> Result<Option<StateId>>;
    fn compute_trs(&self, id: usize) -> Result<TrsVec<W>>;
    fn compute_final_weight(&self, id: StateId) -> Result<Option<W>>;

    // Computed at construction time
    fn properties(&self) -> FstProperties;
}

impl<W: Semiring, F: FstOp<W>> FstOp<W> for Arc<F> {
    fn compute_start(&self) -> Result<Option<StateId>> {
        self.deref().compute_start()
    }

    fn compute_trs(&self, id: usize) -> Result<TrsVec<W>> {
        self.deref().compute_trs(id)
    }

    fn compute_final_weight(&self, id: StateId) -> Result<Option<W>> {
        self.deref().compute_final_weight(id)
    }

    // Computed at construction time
    fn properties(&self) -> FstProperties {
        self.deref().properties()
    }
}

impl<W: Semiring, F: FstOp<W>> FstOp<W> for Rc<F> {
    fn compute_start(&self) -> Result<Option<StateId>> {
        self.deref().compute_start()
    }

    fn compute_trs(&self, id: usize) -> Result<TrsVec<W>> {
        self.deref().compute_trs(id)
    }

    fn compute_final_weight(&self, id: StateId) -> Result<Option<W>> {
        self.deref().compute_final_weight(id)
    }

    // Computed at construction time
    fn properties(&self) -> FstProperties {
        self.deref().properties()
    }
}
