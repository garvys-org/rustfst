use self::super::FilterState;
use num_traits::Num;

/// Filter state that is a signed integral type.
#[derive(Debug, PartialEq, Clone)]
pub struct IntegerFilterState<T: Num> {
    state: Option<T>,
}

impl<T: Num> FilterState for IntegerFilterState<T> {
    type Type = T;

    fn new(value: Self::Type) -> Self {
        Self { state: Some(value) }
    }

    fn state(&self) -> Option<&Self::Type> {
        self.state.as_ref()
    }
}

impl<T: Num> Default for IntegerFilterState<T> {
    fn default() -> Self {
        Self { state: None }
    }
}

pub type IntFilterState = IntegerFilterState<i32>;
pub type ShortFilterState = IntegerFilterState<i16>;
pub type CharFilterState = IntegerFilterState<i8>;
