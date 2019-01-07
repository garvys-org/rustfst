//! # Rustfst
//!
//! Rust implementation of Weighted Finite States Transducers.
//!
//! Rustfst is a library for constructing, combining, optimizing, and searching weighted
//! finite-state transducers (FSTs). Weighted finite-state transducers are automata where
//! each transition has an input label, an output label, and a weight.
//! The more familiar finite-state acceptor is represented as a transducer
//! with each transition's input and output label equal. Finite-state acceptors
//! are used to represent sets of strings (specifically, regular or rational sets);
//! finite-state transducers are used to represent binary relations between pairs of
//! strings (specifically, rational transductions). The weights can be used to represent
//! the cost of taking a particular transition.
//!
//! FSTs have key applications in speech recognition and synthesis, machine translation,
//! optical character recognition, pattern matching, string processing, machine learning,
//! information extraction and retrieval among others. Often a weighted transducer is used to
//! represent a probabilistic model (e.g., an n-gram model, pronunciation model). FSTs can be
//! optimized by determinization and minimization, models can be applied to hypothesis sets
//! (also represented as automata) or cascaded by finite-state composition, and the best
//! results can be selected by shortest-path algorithms.
//!
//! ## References
//!
//! Implementation heavily inspired from Mehryar Mohri's, Cyril Allauzen's and Michael Riley's work :
//! - [Weighted automata algorithms](https://cs.nyu.edu/~mohri/pub/hwa.pdf)
//! - [The design principles of a weighted finite-state transducer library](https://core.ac.uk/download/pdf/82101846.pdf)
//! - [OpenFst: A general and efficient weighted finite-state transducer library](https://link.springer.com/chapter/10.1007%2F978-3-540-76336-9_3)
//! - [Weighted finite-state transducers in speech recognition](https://repository.upenn.edu/cgi/viewcontent.cgi?article=1010&context=cis_papers)
//!
//! ## Example
//!
//! ```
//! # use rustfst::utils::transducer;
//! # use rustfst::semirings::{Semiring, IntegerWeight};
//! # use rustfst::fst_impls::VectorFst;
//! # use rustfst::fst_traits::{MutableFst, PathsIterator};
//! # use rustfst::Arc;
//! // Creates a empty wFST
//! let mut fst = VectorFst::new();
//!
//! // Add some states
//! let s0 = fst.add_state();
//! let s1 = fst.add_state();
//! let s2 = fst.add_state();
//!
//! // Set s0 as the start state
//! fst.set_start(s0).unwrap();
//!
//! // Add an arc from s0 to s1
//! fst.add_arc(s0, Arc::new(3, 5, IntegerWeight::new(10), s1))
//!     .unwrap();
//!
//! // Add an arc from s0 to s2
//! fst.add_arc(s0, Arc::new(5, 7, IntegerWeight::new(18), s2))
//!     .unwrap();
//!
//! // Set s1 and s2 as final states
//! fst.set_final(s1, IntegerWeight::new(31)).unwrap();
//! fst.set_final(s2, IntegerWeight::new(45)).unwrap();
//!
//! // Iter over all the paths in the wFST
//! for p in fst.paths_iter() {
//!     println!("{:?}", p);
//! }
//!
//! ```
//!
//! ## Status
//!
//! Not all algorithms are (yet) implemented, this is still work in progress.
//!

#[warn(missing_docs)]
#[cfg(test)]
extern crate counter;
#[macro_use]
extern crate failure;
extern crate itertools;
#[macro_use]
extern crate nom;
#[cfg(test)]
extern crate rand;
#[cfg(test)]
extern crate serde;
#[cfg(test)]
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
extern crate serde_json;

/// A type alias for dealing with errors returned by this crate.
pub type Result<T> = std::result::Result<T, failure::Error>;

mod symbol_table;
pub use crate::symbol_table::SymbolTable;

/// Type used for the input label and output label of an arc in a wFST -> usize
pub type Label = usize;
/// Symbol to map in the Symbol Table -> String
pub type Symbol = String;

/// Type used to identify a state in a wFST -> usize
pub type StateId = usize;

/// Epsilon label representing the epsilon transition (empty transition) = `0`.
pub const EPS_LABEL: Label = 0;
/// Epsilon symbol representing the epsilon transition (empty transition) = `<eps>`.
pub const EPS_SYMBOL: &str = "<eps>";

/// A few utilities to manipulate wFSTs.
pub mod utils;

/// Provides algorithms that are generic to all wFST.
pub mod algorithms;

/// Implementation of the transitions inside a wFST.
mod arc;
pub use self::arc::Arc;

#[macro_use]
/// Provides traits that must be implemented to be able to use generic algorithms.
pub mod fst_traits;
/// Implementation of the wFST traits with different data structures.
pub mod fst_impls;
/// Provides a trait that shall be implemented for all weights stored inside a wFST.
pub mod semirings;
#[cfg(test)]
pub(crate) mod test_data;

mod drawing_config;
pub use crate::drawing_config::DrawingConfig;

/// Implementation of a successful path inside a wFST.
mod fst_path;
pub use crate::fst_path::FstPath;

mod parsers;

/// A representable float near .001. (Used in Quantize)
pub(crate) const KDELTA: f32 = 1.0f32 / 1024.0f32;
