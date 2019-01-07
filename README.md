# Rustfst

[![Build Status](https://travis-ci.com/Garvys/rustfst.svg?branch=master)](https://travis-ci.com/Garvys/rustfst)
[![Current version](https://meritbadge.herokuapp.com/rustfst)](https://crates.io/crates/rustfst)
[![Documentation](https://docs.rs/rustfst/badge.svg)](https://docs.rs/rustfst)
[![License: MIT/Apache-2.0](https://img.shields.io/crates/l/rustfst.svg)](#license)

Rust implementation of Weighted Finite States Transducers.

Rustfst is a library for constructing, combining, optimizing, and searching weighted
finite-state transducers (FSTs). Weighted finite-state transducers are automata where
each transition has an input label, an output label, and a weight.
The more familiar finite-state acceptor is represented as a transducer
with each transition's input and output label equal. Finite-state acceptors
are used to represent sets of strings (specifically, regular or rational sets);
finite-state transducers are used to represent binary relations between pairs of
strings (specifically, rational transductions). The weights can be used to represent
the cost of taking a particular transition.

FSTs have key applications in speech recognition and synthesis, machine translation,
optical character recognition, pattern matching, string processing, machine learning,
information extraction and retrieval among others. Often a weighted transducer is used to
represent a probabilistic model (e.g., an n-gram model, pronunciation model). FSTs can be
optimized by determinization and minimization, models can be applied to hypothesis sets
(also represented as automata) or cascaded by finite-state composition, and the best
results can be selected by shortest-path algorithms.

## References

Implementation heavily inspired from Mehryar Mohri's, Cyril Alluzen's and Michael Riley's work :
 - [Weighted automata algorithms](https://cs.nyu.edu/~mohri/pub/hwa.pdf)
 - [The design principles of a weighted finite-state transducer library](https://core.ac.uk/download/pdf/82101846.pdf)
 - [OpenFst: A general and efficient weighted finite-state transducer library](https://link.springer.com/chapter/10.1007%2F978-3-540-76336-9_3)
 - [Weighted finite-state transducers in speech recognition](https://repository.upenn.edu/cgi/viewcontent.cgi?article=1010&context=cis_papers)

## Installation

Add it to your `Cargo.toml`:

```
[dependencies]
rustfst = "*"
```

Add `extern crate rustfst` to your crate root and you are good to go!

## Example

```rust
extern crate rustfst;

use rustfst::utils::transducer;
use rustfst::semirings::{Semiring, IntegerWeight};
use rustfst::fst_impls::VectorFst;
use rustfst::fst_traits::{MutableFst, PathsIterator};
use rustfst::Arc;

fn main() {
    // Creates a empty wFST
    let mut fst = VectorFst::new();
    
    // Add some states
    let s0 = fst.add_state();
    let s1 = fst.add_state();
    let s2 = fst.add_state();
    
    // Set s0 as the start state
    fst.set_start(s0).unwrap();
    
    // Add an arc from s0 to s1
    fst.add_arc(s0, Arc::new(3, 5, IntegerWeight::new(10), s1))
         .unwrap();
    
    // Add an arc from s0 to s2
    fst.add_arc(s0, Arc::new(5, 7, IntegerWeight::new(18), s2))
         .unwrap();
    
    // Set s1 and s2 as final states
    fst.set_final(s1, IntegerWeight::new(31)).unwrap();
    fst.set_final(s2, IntegerWeight::new(45)).unwrap();
    
    // Iter over all the paths in the wFST
    for p in fst.paths_iter() {
         println!("{:?}", p);
    }
}
```

## Documentation

The documentation of the last released version is available here :
https://docs.rs/rustfst

## Status

Not all the algorithms are (yet) implemented. This is work in progress.

## License
   
Licensed under either of
- Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.