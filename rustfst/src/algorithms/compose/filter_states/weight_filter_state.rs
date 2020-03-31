use self::super::FilterState;
use crate::semirings::Semiring;

/// Filter state that is a weight implementing the Semiring trait.
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct WeightFilterState<W> {
    state: W,
}

impl<W: Semiring> FilterState for WeightFilterState<W> {
    type Type = W;

    fn new(value: Self::Type) -> Self {
        Self { state: value }
    }

    fn new_no_state() -> Self {
        Self { state: W::zero() }
    }

    fn state(&self) -> &Self::Type {
        &self.state
    }
}
