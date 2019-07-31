use crate::semirings::Semiring;
use crate::{Arc, StateId};

#[derive(Debug, PartialEq, Clone)]
pub struct ConstFst<W: Semiring> {
    pub(crate) states: Vec<ConstState<W>>,
    pub(crate) arcs: Vec<Arc<W>>,
    pub(crate) start: Option<StateId>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct ConstState<W: Semiring> {
    /// Final Weight
    pub(crate) final_weight: Option<W>,
    /// Start of state's arcs in `arcs`.
    pub(crate) pos: usize,
    /// Number of arcs (per state).
    pub(crate) narcs: usize
}
