mod compute_fst_properties;
mod mutate_properties;
pub(crate) mod properties;
mod utils;

/// Functions for getting property bit vectors when executing mutation operations.
pub mod mutable_properties {
    pub use super::mutate_properties::*;
}

pub use self::compute_fst_properties::compute_fst_properties;
pub use self::properties::FstProperties;
pub use self::utils::{compat_properties, known_properties};
