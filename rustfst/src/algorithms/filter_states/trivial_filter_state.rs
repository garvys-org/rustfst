use self::super::FilterState;

/// Single non-blocking filter state.
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct TrivialFilterState {
    state: bool,
}

impl FilterState for TrivialFilterState {
    type Type = bool;

    fn new(value: Self::Type) -> Self {
        Self { state: value }
    }

    fn new_no_state() -> Self {
        Self::new(false)
    }

    fn state(&self) -> &Self::Type {
        &self.state
    }
}
