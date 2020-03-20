use num_traits::Num;

use self::super::FilterState;
use std::fmt::Debug;
use std::hash::Hash;

/// Filter state is a list of signed integer types T. Order matters for equality.
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct ListFilterState<T: Num> {
    state: Option<Vec<T>>,
}

impl<T: Num + Clone + Eq + Hash + Debug> FilterState for ListFilterState<T> {
    type Type = Vec<T>;

    fn new(value: Self::Type) -> Self {
        Self { state: Some(value) }
    }

    fn new_no_state() -> Self {
        unimplemented!()
    }

    fn state(&self) -> &Self::Type {
        unimplemented!()
    }
}
