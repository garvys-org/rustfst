mod compute_fst_properties;
mod fst_properties;
mod mutate_properties;
mod utils;

/// Functions for getting property bit vectors when executing mutation operations.
pub mod mutable_properties {
    pub use super::mutate_properties::*;
}

pub use self::compute_fst_properties::compute_fst_properties;
pub use self::fst_properties::FstProperties;
pub use self::utils::{compat_properties, known_properties};
