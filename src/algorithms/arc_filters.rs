use crate::semirings::Semiring;
use crate::Arc;
use crate::EPS_LABEL;

/// Base trait to restrict which arcs are traversed in an FST.
pub trait ArcFilter<S: Semiring> {
    /// If true, Arc should be kept, else Arc should be ignored.
    fn keep(&self, arc: &Arc<S>) -> bool;
}

/// True for all arcs.
pub struct AnyArcFilter {}

impl<S: Semiring> ArcFilter<S> for AnyArcFilter {
    fn keep(&self, _arc: &Arc<S>) -> bool {
        true
    }
}

/// True for (input/output) epsilon arcs.
pub struct EpsilonArcFilter {}

impl<S: Semiring> ArcFilter<S> for EpsilonArcFilter {
    fn keep(&self, arc: &Arc<S>) -> bool {
        arc.ilabel == EPS_LABEL && arc.olabel == EPS_LABEL
    }
}

/// True for input epsilon arcs.
pub struct InputEpsilonArcFilter {}

impl<S: Semiring> ArcFilter<S> for InputEpsilonArcFilter {
    fn keep(&self, arc: &Arc<S>) -> bool {
        arc.ilabel == EPS_LABEL
    }
}

/// True for output epsilon arcs.
pub struct OutputEpsilonArcFilter {}

impl<S: Semiring> ArcFilter<S> for OutputEpsilonArcFilter {
    fn keep(&self, arc: &Arc<S>) -> bool {
        arc.olabel == EPS_LABEL
    }
}