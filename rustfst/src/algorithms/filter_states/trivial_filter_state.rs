use self::super::FilterState;

/// Single non-blocking filter state.
#[derive(Debug, PartialEq)]
pub struct TrivialFilterState {
    state: bool,
}

impl FilterState for TrivialFilterState {
    type Type = bool;

    fn new(value: Self::Type) -> Self {
        Self { state: value }
    }

    fn state(&self) -> Option<&Self::Type> {
        Some(&self.state)
    }
}

impl Default for TrivialFilterState {
    fn default() -> Self {
        Self { state: false }
    }
}
