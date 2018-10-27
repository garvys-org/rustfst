#[cfg(test)]
extern crate counter;
extern crate failure;
extern crate itertools;
#[cfg(test)]
extern crate rand;

type Result<T> = std::result::Result<T, failure::Error>;

/// Type used for the input label and output label of an arc in a wFST.
pub type Label = usize;

/// Type used to identify a state in a wFST.
pub type StateId = usize;

/// Epsilon label representing the epsilon transition (empty transition).
pub static EPS_LABEL: Label = 0;

/// Provides algorithms that are generic for all wFST.
pub mod algorithms;
/// Implementation of the transitions inside a wFST.
pub mod arc;
/// Implementation of a successful path inside a wFST.
pub mod path;
#[macro_use]
/// Provides trait that must be implemented to be able to use generic algorithms.
pub mod fst_traits;
/// Implementation of the wFST traits with different data structure.
pub mod fst_impls;
/// Provides a trait that shall be implemented for all weights stored inside a wFST.
pub mod semirings;
#[cfg(test)]
pub(crate) mod test_data;
/// A few utilities to manipulate wFSTs.
pub mod utils;
