use crate::semirings::Semiring;
use crate::{Arc, StateId};

/// Immutable FST whose states and arcs each implemented by single arrays,
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
    pub(crate) narcs: usize,
    /// Number of input epsilons
    pub(crate) niepsilons: usize,
    /// Number of output epsilons
    pub(crate) noepsilons: usize,
}
