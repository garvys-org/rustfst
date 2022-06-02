use crate::StateId;
use std::rc::Rc;

/// Random path state info maintained by RandGenFst and passed to samplers.
#[derive(Debug, Clone)]
pub struct RandState {
    /// Current input FST state.
    pub state_id: StateId,
    /// Number of samples to be sampled at this state.
    pub nsamples: usize,
    /// Length of path to this random state.
    pub length: usize,
    /// Previous sample arc selection.
    pub select: usize,
    /// Previous random state on this path.
    pub parent: Option<Rc<RandState>>,
}

impl RandState {
    pub fn new(state_id: StateId) -> Self {
        Self {
            state_id,
            nsamples: 0,
            length: 0,
            select: 0,
            parent: None,
        }
    }

    pub fn with_nsamples(self, nsamples: usize) -> Self {
        Self { nsamples, ..self }
    }

    pub fn with_length(self, length: usize) -> Self {
        Self { length, ..self }
    }

    pub fn with_select(self, select: usize) -> Self {
        Self { select, ..self }
    }

    pub fn with_parent(self, parent: Option<Rc<Self>>) -> Self {
        Self { parent, ..self }
    }
}
