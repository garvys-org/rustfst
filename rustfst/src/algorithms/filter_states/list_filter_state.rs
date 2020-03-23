use std::fmt::Debug;
use std::hash::Hash;

use self::super::FilterState;

/// Filter state is a list of signed integer types T. Order matters for equality.
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct ListFilterState {
    state: Vec<usize>,
}

impl FilterState for ListFilterState {
    type Type = Vec<usize>;

    fn new(value: Self::Type) -> Self {
        unimplemented!()
    }

    fn new_no_state() -> Self {
        unimplemented!()
    }

    fn state(&self) -> &Self::Type {
        unimplemented!()
    }
}
