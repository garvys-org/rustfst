use std::cmp::{Eq, PartialEq};

/// This enumeration represents the computation status in a cache.
#[derive(Debug, Clone, Copy)]
pub enum CacheStatus<T> {
    NotComputed,
    Computed(T),
}

impl<T> CacheStatus<T> {
    /// Map the computed value and returned it as a cache status.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> CacheStatus<U> {
        match self {
            CacheStatus::Computed(x) => CacheStatus::Computed((f)(x)),
            CacheStatus::NotComputed => CacheStatus::NotComputed,
        }
    }

    /// Returns `NotComputed` if self is not computed otherwise it returns other.
    pub fn and<U>(self, other: CacheStatus<U>) -> CacheStatus<U> {
        if self.is_not_computed() {
            CacheStatus::NotComputed
        } else {
            other
        }
    }

    /// Chain lazyly two `CacheStatus` and flatten them.
    pub fn and_then<U>(self, f: impl FnOnce(T) -> CacheStatus<U>) -> CacheStatus<U> {
        match self {
            CacheStatus::Computed(x) => (f)(x),
            CacheStatus::NotComputed => CacheStatus::NotComputed,
        }
    }

    /// Convert into an `Option`. `Some` if computed otherwise `None`.
    pub fn into_option(self) -> Option<T> {
        match self {
            CacheStatus::Computed(e) => Some(e),
            CacheStatus::NotComputed => None,
        }
    }

    /// Convert to an `Option`. `Some` if computed otherwise `None`.
    pub fn to_option(&self) -> Option<&T> {
        match self {
            CacheStatus::Computed(e) => Some(e),
            CacheStatus::NotComputed => None,
        }
    }

    /// Return true if `CachStatus` is computed.
    pub fn is_computed(&self) -> bool {
        matches!(self, Self::Computed(_))
    }

    /// Return true if `CachStatus` is not computed.
    pub fn is_not_computed(&self) -> bool {
        matches!(self, Self::NotComputed)
    }

    /// If the value is already computed, self is returned otherwise `other` is returned.
    ///
    /// If a lazy implementation is preferable, call `or_else`.
    pub fn or(self, other: CacheStatus<T>) -> CacheStatus<T> {
        match self {
            Self::Computed(v) => Self::Computed(v),
            Self::NotComputed => other,
        }
    }

    // If the value is already computer, self is returned otherwise the `f` result will be
    // computed and returned.
    pub fn or_else(self, f: impl FnOnce() -> CacheStatus<T>) -> CacheStatus<T> {
        match self {
            Self::Computed(v) => Self::Computed(v),
            Self::NotComputed => (f)(),
        }
    }
}

impl<T: Eq> PartialEq for CacheStatus<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Computed(lhs), Self::Computed(rhs)) => lhs == rhs,
            (Self::NotComputed, Self::NotComputed) => true,
            (Self::Computed(_), Self::NotComputed) | (Self::NotComputed, Self::Computed(_)) => {
                false
            }
        }
    }
}

impl<T> From<Option<T>> for CacheStatus<T> {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(v) => Self::Computed(v),
            None => Self::NotComputed,
        }
    }
}
