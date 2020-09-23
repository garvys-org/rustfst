#[derive(Debug, Clone, Copy)]
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

    pub fn into_option(self) -> Option<T> {
        match self {
            CacheStatus::Computed(e) => Some(e),
            CacheStatus::NotComputed => None,
        }
    }
}
