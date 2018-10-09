#[cfg(test)]
extern crate rand;
#[macro_use]
extern crate failure;

type Result<T> = std::result::Result<T, failure::Error>;

/// Type used for the input label and output label of an arc in a wFST
pub type Label = usize;

/// Type used to identify a state in a wFST
pub type StateId = usize;

pub mod algorithms;
pub mod arc;
#[macro_use]
pub mod fst_traits;
pub mod fst_impls;
pub mod semirings;
pub mod utils;
