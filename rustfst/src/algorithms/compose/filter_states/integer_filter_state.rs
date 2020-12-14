use std::fmt::Debug;
use std::hash::Hash;

use crate::{StateId, NO_STATE_ID};

use self::super::FilterState;

/// Filter state that is a signed integral type.
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct IntegerFilterState {
    state: StateId,
}

impl FilterState for IntegerFilterState {
    type Type = StateId;

    fn new(value: Self::Type) -> Self {
        Self { state: value }
    }
    fn new_no_state() -> Self {
        Self { state: NO_STATE_ID }
    }

    fn state(&self) -> &Self::Type {
        &self.state
    }
}

// pub type IntFilterState = IntegerFilterState<i32>;
// pub type ShortFilterState = IntegerFilterState<i16>;
// pub type CharFilterState = IntegerFilterState<i8>;
