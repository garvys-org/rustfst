use std::cell::{Ref, RefCell};
use std::fmt;
use std::hash::Hash;

use bimap::BiHashMap;

use crate::StateId;

#[derive(Clone, Eq)]
pub struct StateTable<T: Hash + Eq + Clone> {
    pub(crate) table: RefCell<BiHashMap<StateId, T>>,
}

impl<T: Hash + Eq + Clone + fmt::Debug> fmt::Debug for StateTable<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StateTable {{ table : {:?} }}", self.table.borrow())
    }
}

impl<T: Hash + Eq + Clone + PartialEq> PartialEq for StateTable<T> {
    fn eq(&self, other: &Self) -> bool {
        self.table.borrow().eq(&*other.table.borrow())
    }
}

impl<T: Hash + Eq + Clone> StateTable<T> {
    pub fn new() -> Self {
        Self {
            table: RefCell::new(BiHashMap::new()),
        }
    }

    /// Looks up integer ID from entry. If it doesn't exist and insert
    pub fn find_id_from_ref(&self, tuple: &T) -> StateId {
        if !self.table.borrow().contains_right(tuple) {
            let n = self.table.borrow().len();
            self.table.borrow_mut().insert(n, tuple.clone());
            return n;
        }
        *self.table.borrow().get_by_right(tuple).unwrap()
    }

    pub fn find_id(&self, tuple: T) -> StateId {
        if !self.table.borrow().contains_right(&tuple) {
            let n = self.table.borrow().len();
            self.table.borrow_mut().insert(n, tuple);
            return n;
        }
        *self.table.borrow().get_by_right(&tuple).unwrap()
    }

    /// Looks up tuple from integer ID.
    pub fn find_tuple(&self, tuple_id: StateId) -> Ref<T> {
        let table = self.table.borrow();
        Ref::map(table, |x| x.get_by_left(&tuple_id).unwrap())
    }
}
