use crate::semirings::Semiring;
use crate::{Label, StateId};

/// Structure representing a transition from a state to another state in a FST.
#[derive(Debug, Clone, PartialEq)]
pub struct Arc<W: Semiring> {
    /// Input label.
    pub ilabel: Label,
    /// Output label.
    pub olabel: Label,
    /// Weight.
    pub weight: W,
    /// ID of the next state.
    pub nextstate: StateId,
}

impl<W: Semiring> Arc<W> {
    /// Creates a new Arc.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::Arc;
    /// # use rustfst::semirings::{BooleanWeight, Semiring};
    /// let arc = Arc::new(0, 1, BooleanWeight::ONE, 2);
    ///
    /// assert_eq!(arc.ilabel, 0);
    /// assert_eq!(arc.olabel, 1);
    /// assert_eq!(arc.weight, BooleanWeight::ONE);
    /// assert_eq!(arc.nextstate, 2);
    ///
    /// ```
    pub fn new(ilabel: Label, olabel: Label, weight: W, nextstate: StateId) -> Self {
        Arc {
            ilabel,
            olabel,
            weight,
            nextstate,
        }
    }

    /// Updates the values of the attributes of the Arc from another Arc.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::Arc;
    /// # use rustfst::semirings::{BooleanWeight, Semiring};
    /// let mut arc_1 = Arc::new(0, 1, BooleanWeight::ONE, 2);
    /// let arc_2 = Arc::new(1, 2, BooleanWeight::ZERO, 3);
    ///
    /// arc_1.set_value(&arc_2);
    ///
    /// assert_eq!(arc_1, arc_2);
    /// ```
    pub fn set_value(&mut self, arc: &Arc<W>) {
        self.ilabel = arc.ilabel;
        self.olabel = arc.olabel;
        self.weight = arc.weight;
        self.nextstate = arc.nextstate;
    }
}
