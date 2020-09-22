use crate::StateId;
use std::collections::HashMap;

pub type StartState = Option<StateId>;
pub type FinalWeight<W> = Option<W>;

#[derive(Debug)]
pub struct CachedData<T> {
    pub data: T,
    pub num_known_states: usize,
}

impl<T> Default for CachedData<Option<T>> {
    fn default() -> Self {
        Self {
            data: None,
            num_known_states: 0,
        }
    }
}

impl<T> CachedData<Option<T>> {
    pub fn clear(&mut self) {
        self.data.take();
        self.num_known_states = 0;
    }
}

impl<T> Default for CachedData<Vec<T>> {
    fn default() -> Self {
        Self {
            data: vec![],
            num_known_states: 0,
        }
    }
}

impl<T> CachedData<Vec<T>> {
    pub fn clear(&mut self) {
        self.data.clear();
        self.num_known_states = 0;
    }
}

impl<K, V> Default for CachedData<HashMap<K, V>> {
    fn default() -> Self {
        Self {
            data: HashMap::new(),
            num_known_states: 0,
        }
    }
}

impl<K, V> CachedData<HashMap<K, V>> {
    pub fn clear(&mut self) {
        self.data.clear();
        self.num_known_states = 0;
    }
}

impl<T: Clone> Clone for CachedData<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            num_known_states: self.num_known_states,
        }
    }
}
