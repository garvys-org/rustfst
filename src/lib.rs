#[cfg(test)]
extern crate rand;

/// Type used for the input label and output label of an arc in a wFST
pub type Label = usize;

/// Type used to identify a state in a wFST
pub type StateId = usize;

pub mod algorithms;
pub mod arc;
pub mod fst_impls;
pub mod fst_traits;
pub mod semirings;
pub mod utils;
