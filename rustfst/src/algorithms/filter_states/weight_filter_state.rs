use self::super::FilterState;
use crate::semirings::Semiring;

/// Filter state that is a weight implementing the Semiring trait.
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct WeightFilterState<W: Semiring> {
    state: Option<W>,
}

impl<W: Semiring> FilterState for WeightFilterState<W> {
    type Type = W;

    fn new(value: Self::Type) -> Self {
        Self { state: Some(value) }
    }

    fn new_no_state() -> Self {
        unimplemented!()
    }

    fn state(&self) -> Option<&Self::Type> {
        self.state.as_ref()
    }
}

impl<W: Semiring> Default for WeightFilterState<W> {
    fn default() -> Self {
        Self { state: None }
    }
}
