use std::fmt::Debug;

use anyhow::Result;

use crate::semirings::Semiring;
use crate::{StateId, TrsVec};

pub trait FstOp<W: Semiring>: Debug {
    // was FstImpl
    fn compute_start(&self) -> Result<Option<StateId>>;
    fn compute_trs(&self, id: usize) -> Result<TrsVec<W>>;
    fn compute_final_weight(&self, id: StateId) -> Result<Option<W>>;
}
