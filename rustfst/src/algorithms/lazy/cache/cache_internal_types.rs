use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;

use crate::algorithms::lazy::CacheStatus;
use crate::semirings::Semiring;
use crate::{StateId, TrsVec};

pub type StartState = Option<StateId>;
pub type FinalWeight<W> = Option<W>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheTrs<W: Semiring> {
    pub trs: TrsVec<W>,
    pub niepsilons: usize,
    pub noepsilons: usize,
}

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

impl<T> Default for CachedData<CacheStatus<T>> {
    fn default() -> Self {
        Self {
            data: CacheStatus::NotComputed,
            num_known_states: 0,
        }
    }
}

impl<T> CachedData<CacheStatus<T>> {
    pub fn clear(&mut self) {
        self.data = CacheStatus::NotComputed;
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

impl<T> CachedData<Vec<CacheStatus<T>>> {
    pub fn get(&self, idx: StateId) -> CacheStatus<&T> {
        match self.data.get(idx as usize) {
            Some(e) => match e {
                CacheStatus::Computed(v) => CacheStatus::Computed(v),
                CacheStatus::NotComputed => CacheStatus::NotComputed,
            },
            None => CacheStatus::NotComputed,
        }
    }
}

impl<K: Hash + Eq, V> Default for CachedData<HashMap<K, V>> {
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

impl<K: Hash + Eq, V: Semiring> CachedData<HashMap<K, CacheTrs<V>>> {
    pub fn get(&self, idx: K) -> Option<&CacheTrs<V>> {
        self.data.get(&idx)
    }
}

impl<K: Hash + Eq, W: Semiring> CachedData<HashMap<K, FinalWeight<W>>> {
    pub fn get(&self, idx: K) -> Option<&FinalWeight<W>> {
        self.data.get(&idx)
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
