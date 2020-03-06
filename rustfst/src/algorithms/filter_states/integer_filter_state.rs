use self::super::FilterState;
use num_traits::Num;
use std::hash::Hash;
use std::fmt::Debug;

/// Filter state that is a signed integral type.
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct IntegerFilterState<T: Num> {
    state: Option<T>,
}

impl<T: Num + Clone + Eq + Hash + Debug> FilterState for IntegerFilterState<T> {
    type Type = T;

    fn new(value: Self::Type) -> Self {
        Self { state: Some(value) }
    }

    fn state(&self) -> Option<&Self::Type> {
        self.state.as_ref()
    }
}

impl<T: Num + Clone + Eq + Hash> Default for IntegerFilterState<T> {
    fn default() -> Self {
        Self { state: None }
    }
}

pub type IntFilterState = IntegerFilterState<i32>;
pub type ShortFilterState = IntegerFilterState<i16>;
pub type CharFilterState = IntegerFilterState<i8>;
