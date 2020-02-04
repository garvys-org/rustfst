use num_traits::Num;

use self::super::FilterState;

/// Filter state is a list of signed integer types T. Order matters for equality.
#[derive(Debug, PartialEq)]
pub struct ListFilterState<T: Num> {
    state: Option<Vec<T>>,
}

impl<T: Num> FilterState for ListFilterState<T> {
    type Type = Vec<T>;

    fn new(value: Self::Type) -> Self {
        Self { state: Some(value) }
    }

    fn state(&self) -> Option<&Self::Type> {
        self.state.as_ref()
    }
}

impl<T: Num> Default for ListFilterState<T> {
    fn default() -> Self {
        Self { state: None }
    }
}
