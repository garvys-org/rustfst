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
//! ![fst](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/project_in.svg?sanitize=true)
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
//! ```rust
//! use anyhow::Result;
//! use rustfst::prelude::*;
//! use rustfst::algorithms::determinize::{DeterminizeType, determinize};
//! use rustfst::algorithms::rm_epsilon::rm_epsilon;
//! use std::sync::Arc;
//!
//! fn main() -> Result<()> {
//!     // Creates a empty wFST
//!     let mut fst = VectorFst::<TropicalWeight>::new();
//!
//!     // Add some states
//!     let s0 = fst.add_state();
//!     let s1 = fst.add_state();
//!     let s2 = fst.add_state();
//!
//!     // Set s0 as the start state
//!     fst.set_start(s0)?;
//!
//!     // Add a transition from s0 to s1
//!     fst.add_tr(s0, Tr::new(3, 5, 10.0, s1))?;
//!
//!     // Add a transition from s0 to s2
//!     fst.add_tr(s0, Tr::new(5, 7, 18.0, s2))?;
//!
//!     // Set s1 and s2 as final states
//!     fst.set_final(s1, 31.0)?;
//!     fst.set_final(s2, 45.0)?;
//!
//!     // Iter over all the paths in the wFST
//!     for p in fst.paths_iter() {
//!          println!("{:?}", p);
//!     }
//!
//!     // A lot of operations are available to modify/optimize the FST.
//!     // Here are a few examples :
//!
//!     // - Remove useless states.
//!     connect(&mut fst)?;
//!
//!     // - Optimize the FST by merging states with the same behaviour.
//!     minimize(&mut fst, true)?;
//!
//!     // - Copy all the input labels in the output.
//!     project(&mut fst, ProjectType::ProjectInput);
//!
//!     // - Remove epsilon transitions.
//!     rm_epsilon(&mut fst)?;
//!
//!     // - Compute an equivalent FST but deterministic.
//!     fst = determinize(Arc::new(fst), DeterminizeType::DeterminizeFunctional)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Status
//!
//! A big number of algorithms are already implemented. The main one missing is the Composition.

#[warn(missing_docs)]
#[cfg(test)]
extern crate counter;
#[macro_use]
extern crate anyhow;
#[cfg(test)]
extern crate rand;
#[cfg(test)]
extern crate serde;
#[cfg(test)]
extern crate serde_json;

pub use crate::drawing_config::DrawingConfig;
pub use crate::fst_path::{check_path_in_fst, FstPath};
pub use crate::symbol_table::SymbolTable;

pub use self::tr::Tr;
pub use self::trs::{Trs, TrsConst, TrsVec};

pub use crate::semirings::Semiring;
#[cfg(test)]
use doc_comment::doc_comment;

// When running `cargo test`, rustdoc will check this file as well.
#[cfg(test)]
doc_comment!(include_str!("../../README.md"));

#[cfg(test)]
mod tests_openfst;

mod symbol_table;

/// Type used for the input label and output label of a transition in a wFST -> usize
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

/// Provides the `FstProperties` struct and some utils functions around it.
/// Useful to assert some properties on a Fst.
pub mod fst_properties;
/// Implementation of the transitions inside a wFST.
mod tr;

#[macro_use]
/// Provides traits that must be implemented to be able to use generic algorithms.
pub mod fst_traits;
/// Implementation of the wFST traits with different data structures.
pub mod fst_impls;
/// Provides a trait that shall be implemented for all weights stored inside a wFST.
pub mod semirings;

mod drawing_config;
/// Implementation of a successful path inside a wFST.
mod fst_path;
mod parsers;

/// A representable float near .001. (Used in Quantize)
pub const KDELTA: f32 = 1.0f32 / 1024.0f32;

/// Module re-exporting most of the objects from this crate.
pub mod prelude {
    pub use crate::algorithms::tr_compares::*;
    pub use crate::algorithms::*;
    pub use crate::fst_impls::*;
    pub use crate::fst_traits::*;
    pub use crate::semirings::*;
    pub use crate::tr::Tr;
}

mod proptest_fst;

pub(crate) static NO_LABEL: Label = std::usize::MAX;
pub(crate) static NO_STATE_ID: StateId = std::usize::MAX;
pub(crate) static UNASSIGNED: usize = std::usize::MAX;

pub mod trs;
