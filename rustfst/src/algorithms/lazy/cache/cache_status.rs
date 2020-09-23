#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub enum CacheStatus<T> {
    NotComputed,
    Computed(T),
}

impl<T> CacheStatus<T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> CacheStatus<U> {
        match self {
            CacheStatus::Computed(x) => CacheStatus::Computed(f(x)),
            CacheStatus::NotComputed => CacheStatus::NotComputed,
        }
    }

    pub fn ok_or<E>(self, err: E) -> Result<T, E> {
        match self {
            CacheStatus::Computed(v) => Ok(v),
            CacheStatus::NotComputed => Err(err),
        }
    }

    pub fn ok_or_else<E, F: FnOnce() -> E>(self, err: F) -> Result<T, E> {
        match self {
            CacheStatus::Computed(v) => Ok(v),
            CacheStatus::NotComputed => Err(err()),
        }
    }

    pub fn unwrap(self) -> T {
        match self {
            CacheStatus::Computed(e) => e,
            CacheStatus::NotComputed => unreachable!(),
        }
    }

    pub fn into_option(self) -> Option<T> {
        match self {
            CacheStatus::Computed(e) => Some(e),
            CacheStatus::NotComputed => None,
        }
    }

    pub fn is_computed(&self) -> bool {
        match self {
            CacheStatus::Computed(_) => true,
            CacheStatus::NotComputed => false,
        }
    }
}
