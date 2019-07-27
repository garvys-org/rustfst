use crate::arc::Arc;
use crate::semirings::Semiring;
use crate::StateId;

/// Simple concrete, mutable FST whose states and arcs are stored in standard vectors.
///
/// All states are stored in a vector of states.
/// In each state, there is a vector of arcs containing the outgoing transitions.
#[derive(Debug, PartialEq, Clone)]
pub struct VectorFst<W: Semiring> {
    pub(crate) states: Vec<VectorFstState<W>>,
    pub(crate) start_state: Option<StateId>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VectorFstState<W: Semiring> {
    pub(crate) final_weight: Option<W>,
    pub(crate) arcs: Vec<Arc<W>>,
}

impl<W: Semiring> VectorFstState<W> {
    pub fn num_arcs(&self) -> usize {
        self.arcs.len()
    }
}