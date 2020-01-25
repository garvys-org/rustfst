# Rustfst

[![Build Status](https://travis-ci.com/Garvys/rustfst.svg?branch=master)](https://travis-ci.com/Garvys/rustfst)
[![Current version](https://meritbadge.herokuapp.com/rustfst)](https://crates.io/crates/rustfst)
[![Documentation](https://docs.rs/rustfst/badge.svg)](https://docs.rs/rustfst)
[![License: MIT/Apache-2.0](https://img.shields.io/crates/l/rustfst.svg)](#license)
[![Code Coverage](https://codecov.io/gh/Garvys/rustfst/branch/master/graph/badge.svg)](https://codecov.io/gh/Garvys/rustfst/branch/master)
[![Coverage Status](https://coveralls.io/repos/github/Garvys/rustfst/badge.svg?branch=master)](https://coveralls.io/github/Garvys/rustfst?branch=master)

<!-- cargo-sync-readme start -->

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

![fst](https://raw.githubusercontent.com/Garvys/rustfst/task/images_in_doc/rustfst-images-doc/images/project_in.svg?sanitize=true "fst")

## References

Implementation heavily inspired from Mehryar Mohri's, Cyril Allauzen's and Michael Riley's work :
- [Weighted automata algorithms](https://cs.nyu.edu/~mohri/pub/hwa.pdf)
- [The design principles of a weighted finite-state transducer library](https://core.ac.uk/download/pdf/82101846.pdf)
- [OpenFst: A general and efficient weighted finite-state transducer library](https://link.springer.com/chapter/10.1007%2F978-3-540-76336-9_3)
- [Weighted finite-state transducers in speech recognition](https://repository.upenn.edu/cgi/viewcontent.cgi?article=1010&context=cis_papers)

## Example

```rust
use failure::Fallible;
use rustfst::prelude::*;

fn main() -> Fallible<()> {
    // Creates a empty wFST
    let mut fst = VectorFst::new();

    // Add some states
    let s0 = fst.add_state();
    let s1 = fst.add_state();
    let s2 = fst.add_state();

    // Set s0 as the start state
    fst.set_start(s0)?;

    // Add an arc from s0 to s1
    fst.add_arc(s0, Arc::new(3, 5, TropicalWeight::new(10.0), s1))?;

    // Add an arc from s0 to s2
    fst.add_arc(s0, Arc::new(5, 7, TropicalWeight::new(18.0), s2))?;

    // Set s1 and s2 as final states
    fst.set_final(s1, TropicalWeight::new(31.0))?;
    fst.set_final(s2, TropicalWeight::new(45.0))?;

    // Iter over all the paths in the wFST
    for p in fst.paths_iter() {
         println!("{:?}", p);
    }

    // A lot of operations are available to modify/optimize the FST.
    // Here are a few examples :

    // - Remove useless states.
    connect(&mut fst)?;

    // - Optimize the FST by merging states with the same behaviour.
    minimize(&mut fst, true)?;

    // - Copy all the input labels in the output.
    project(&mut fst, ProjectType::ProjectInput);

    // - Remove epsilon transitions.
    fst = rm_epsilon(&fst)?;

    // - Compute an equivalent FST but deterministic.
    fst = determinize(&fst, DeterminizeType::DeterminizeFunctional)?;

    Ok(())
}
```

## Status

A big number of algorithms are already implemented. The main one missing is the Composition.

<!-- cargo-sync-readme end -->

## Documentation

The documentation of the last released version is available here :
https://docs.rs/rustfst

## License
   
Licensed under either of
- Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

