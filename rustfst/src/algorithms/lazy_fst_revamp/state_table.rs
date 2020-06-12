use std::fmt;
use std::hash::Hash;
use std::sync::Mutex;

use crate::StateId;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

#[derive(Clone, Debug, PartialEq)]
pub struct BiHashMap<T: Hash + Eq + Clone> {
    tuple_to_id: HashMap<T, StateId>,
    id_to_tuple: Vec<T>
}

impl<T: Hash + Eq + Clone> BiHashMap<T> {
    pub fn new() -> Self {
        Self {
            tuple_to_id: HashMap::new(),
            id_to_tuple: Vec::new()
        }
    }

    pub fn get_id(&mut self, tuple: T) -> usize {
        match self.tuple_to_id.entry(tuple) {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(e) => {
                let n = self.id_to_tuple.len();
                self.id_to_tuple.push(e.key().clone());
                e.insert(n);
                n
            }
        }
    }

    pub fn get_tuple(&self, id: usize) -> T {
        self.id_to_tuple[id].clone()
    }
}

pub struct StateTable<T: Hash + Eq + Clone> {
    pub(crate) table: Mutex<BiHashMap<T>>,
}

impl<T: Hash + Eq + Clone> Clone for StateTable<T> {
    fn clone(&self) -> Self {
        Self {
            table: Mutex::new(self.table.lock().unwrap().clone()),
        }
    }
}

impl<T: Hash + Eq + Clone> Default for StateTable<T> {
    fn default() -> Self {
        Self {
            table: Mutex::new(BiHashMap::new()),
        }
    }
}

impl<T: Hash + Eq + Clone + fmt::Debug> fmt::Debug for StateTable<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StateTable {{ table : {:?} }}",
            self.table.lock().unwrap()
        )
    }
}

impl<T: Hash + Eq + Clone + PartialEq> PartialEq for StateTable<T> {
    fn eq(&self, other: &Self) -> bool {
        self.table.lock().unwrap().eq(&*other.table.lock().unwrap())
    }
}

impl<T: Hash + Eq + Clone> StateTable<T> {
    pub fn new() -> Self {
        Self {
            table: Mutex::new(BiHashMap::new()),
        }
    }

    // Be carefull with deadlock, this need access to the mutex
    // pub fn insert(&self, tuple: T) -> usize {
    //     let mut table = self.table.lock().unwrap();
    //     let n = table.len();
    //     table.insert(n, tuple);
    //     n
    // }

    /// Looks up integer ID from entry. If it doesn't exist and insert
    pub fn find_id_from_ref(&self, tuple: &T) -> StateId {
        let mut table = self.table.lock().unwrap();
        table.get_id(tuple.clone())
        // if !table.contains_right(tuple) {
        //     let n = table.len();
        //     table.insert(n, tuple.clone());
        //     return n;
        // }
        // *table.get_by_right(tuple).unwrap()
    }

    pub fn find_id(&self, tuple: T) -> StateId {
        let mut table = self.table.lock().unwrap();
        table.get_id(tuple)
    }

    /// Looks up tuple from integer ID.
    pub fn find_tuple(&self, tuple_id: StateId) -> T {
        let table = self.table.lock().unwrap();
        // table.get_by_left(&tuple_id).unwrap().clone()
        table.get_tuple(tuple_id)
    }
}
