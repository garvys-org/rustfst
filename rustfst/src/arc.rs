use crate::semirings::SerializableSemiring;
use crate::{Label, StateId};

/// Structure representing a transition from a state to another state in a FST.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub struct Arc<W> {
    /// Input label.
    pub ilabel: Label,
    /// Output label.
    pub olabel: Label,
    /// Weight.
    pub weight: W,
    /// ID of the next state.
    pub nextstate: StateId,
}

impl<W> Arc<W> {
    /// Creates a new Arc.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::Arc;
    /// # use rustfst::semirings::{TropicalWeight, Semiring};
    /// let arc = Arc::<TropicalWeight>::new(0, 1, 1.3, 2);
    ///
    /// assert_eq!(arc.ilabel, 0);
    /// assert_eq!(arc.olabel, 1);
    /// assert_eq!(arc.weight, TropicalWeight::new(1.3));
    /// assert_eq!(arc.nextstate, 2);
    ///
    /// ```
    pub fn new<S: Into<W>>(ilabel: Label, olabel: Label, weight: S, nextstate: StateId) -> Self {
        Arc {
            ilabel,
            olabel,
            weight: weight.into(),
            nextstate,
        }
    }

    /// Updates the values of the attributes of the Arc from another Arc.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::Arc;
    /// # use rustfst::semirings::{Semiring, TropicalWeight};
    /// let mut arc_1 = Arc::<TropicalWeight>::new(0, 1, 1.3, 2);
    /// let arc_2 = Arc::new(1, 2, 1.2, 3);
    ///
    /// arc_1.set_value(&arc_2);
    ///
    /// assert_eq!(arc_1, arc_2);
    /// ```
    #[inline]
    pub fn set_value(&mut self, arc: &Arc<W>)
    where
        W: std::clone::Clone,
    {
        self.ilabel = arc.ilabel;
        self.olabel = arc.olabel;
        self.weight = arc.weight.clone();
        self.nextstate = arc.nextstate;
    }
}

impl<W: SerializableSemiring> Arc<W> {
    pub fn arc_type() -> String {
        let weight_type = W::weight_type();
        if weight_type.as_str() == "tropical" {
            "standard".to_string()
        } else {
            weight_type
        }
    }
}
