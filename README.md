# Rustfst

[![License: MIT/Apache-2.0](https://img.shields.io/crates/l/rustfst.svg)](#license)
[![Maintenance](https://img.shields.io/badge/Maintained%3F-yes-green.svg)](https://GitHub.com/Naereen/StrapDown.js/graphs/commit-activity)
[![Github tag](https://badgen.net/github/tag/garvys-org/rustfst)](https://github.com/garvys-org/rustfst/tags/)


#### Rust
![rustc >= 1.51.0](https://img.shields.io/badge/rustc-%3E%3D1.51.0-brightgreen)
[![Native Linux test status](https://github.com/garvys-org/rustfst/workflows/Native/badge.svg)](https://github.com/garvys-org/rustfst/actions)
[![Documentation](https://docs.rs/rustfst/badge.svg)](https://docs.rs/rustfst)
[![](https://tokei.rs/b1/github/garvys-org/rustfst)](https://github.com/garvys-org/rustfst)
#### Python
[![PyPI version](https://badge.fury.io/py/rustfst-python.svg)](https://badge.fury.io/py/rustfst-python)
[![PyPI download month](https://img.shields.io/pypi/dm/rustfst-python.svg)](https://pypi.python.org/pypi/rustfst-python/)
[![PyPI pyversions](https://img.shields.io/pypi/pyversions/rustfst-python.svg)](https://pypi.python.org/pypi/rustfst-python/)


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

![fst](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/project_in.svg?sanitize=true)

## Overview

For a basic [example](#example) see the section below.

Some simple and commonly encountered types of FSTs can be easily
created with the macro [`fst`] or the functions
[`acceptor`](utils::acceptor) and
[`transducer`](utils::transducer).

For more complex cases you will likely start with the
[`VectorFst`](fst_impls::VectorFst) type, which will be imported
in the [`prelude`] along with most everything else you need.
[`VectorFst<TropicalWeight>`](fst_impls::VectorFst) corresponds
directly to the OpenFST `StdVectorFst`, and can be used to load
its files using [`read`](fst_traits::SerializableFst::read) or
[`read_text`](fst_traits::SerializableFst::read_text).

Because "iteration" over an FST can mean many different things,
there are a variety of different iterators.  To iterate over state
IDs you may use
[`states_iter`](fst_traits::StateIterator::states_iter), while to
iterate over transitions out of a state, you may use
[`get_trs`](fst_traits::CoreFst::get_trs).  Since it is common to
iterate over both, this can be done using
[`fst_iter`](fst_traits::FstIterator::fst_iter) or
[`fst_into_iter`](fst_traits::FstIntoIterator::fst_into_iter).  It
is also very common to iterate over paths accepted by an FST,
which can be done with
[`paths_iter`](fst_traits::Fst::paths_iter), and as a convenience
for generating text,
[`string_paths_iter`](fst_traits::Fst::string_paths_iter).
Alternately, in the case of a linear FST, you may retrieve the
only possible path with
[`decode_linear_fst`](utils::decode_linear_fst).

Note that iterating over paths is not the same thing as finding
the *shortest* path or paths, which is done with
[`shortest_path`](algorithms::shortest_path) (for a single path)
or
[`shortest_path_with_config`](algorithms::shortest_path_with_config)
(for N-shortest paths).

For the complete list of algorithms, see the [`algorithms`] module.

You may now be wondering, especially if you have previously used
such linguist-friendly tools as
[pyfoma](https://github.com/mhulden/pyfoma), "what if I just want
to *transduce some text*???"  The unfriendly answer is that
rustfst is a somewhat lower-level library, designed for
implementing things like speech recognizers.  The somewhat more
helpful answer is that you would do this by constructing an
[`acceptor`](utils::acceptor) for your input, which you will
[`compose`](algorithms::compose) with a
[`transducer`](utils::transducer), then
[`project`](algorithms::project) the result [to itsoutput](algorithms::ProjectType::ProjectOutput), and finally
[iterate over the paths](fst_traits::Fst::string_paths_iter) in
the resulting FST.

## References

Implementation heavily inspired from Mehryar Mohri's, Cyril Allauzen's and Michael Riley's work :
- [Weighted automata algorithms](https://cs.nyu.edu/~mohri/pub/hwa.pdf)
- [The design principles of a weighted finite-state transducer library](https://core.ac.uk/download/pdf/82101846.pdf)
- [OpenFst: A general and efficient weighted finite-state transducer library](https://link.springer.com/chapter/10.1007%2F978-3-540-76336-9_3)
- [Weighted finite-state transducers in speech recognition](https://repository.upenn.edu/cgi/viewcontent.cgi?article=1010&context=cis_papers)

The API closely resembles that of OpenFST, with some
simplifications and changes to make it more idiomatic in Rust, notably
the use of `Tr` instead of `Arc`.  See [Differences fromOpenFST](#differences-from-openfst) for more information.

## Example

```rust
use anyhow::Result;
use rustfst::prelude::*;
use rustfst::algorithms::determinize::{DeterminizeType, determinize};
use rustfst::algorithms::rm_epsilon::rm_epsilon;

fn main() -> Result<()> {
    // Creates a empty wFST
    let mut fst = VectorFst::<TropicalWeight>::new();

    // Add some states
    let s0 = fst.add_state();
    let s1 = fst.add_state();
    let s2 = fst.add_state();

    // Set s0 as the start state
    fst.set_start(s0)?;

    // Add a transition from s0 to s1
    fst.add_tr(s0, Tr::new(3, 5, 10.0, s1))?;

    // Add a transition from s0 to s2
    fst.add_tr(s0, Tr::new(5, 7, 18.0, s2))?;

    // Set s1 and s2 as final states
    fst.set_final(s1, 31.0)?;
    fst.set_final(s2, 45.0)?;

    // Iter over all the paths in the wFST
    for p in fst.paths_iter() {
         println!("{:?}", p);
    }

    // A lot of operations are available to modify/optimize the FST.
    // Here are a few examples :

    // - Remove useless states.
    connect(&mut fst)?;

    // - Optimize the FST by merging states with the same behaviour.
    minimize(&mut fst)?;

    // - Copy all the input labels in the output.
    project(&mut fst, ProjectType::ProjectInput);

    // - Remove epsilon transitions.
    rm_epsilon(&mut fst)?;

    // - Compute an equivalent FST but deterministic.
    fst = determinize(&fst)?;

    Ok(())
}
```

## Differences from OpenFST

Here is a non-exhaustive list of ways in which Rustfst's API
differs from OpenFST:

- The default epsilon symbol is `<eps>` and not `<epsilon>`.
- Functions and methods follow Rust naming conventions,
  e.g. `add_state` rather than `AddState`, but are otherwise mostly
  equivalent, except that:
- Transitions are called `Tr` and not `Arc`, because `Arc` has a
  rather different and well-established meaning in Rust, and rustfst
  uses it (`std::sync::Arc`, that is) to reference-count symbol
  tables.  All associated functions also use `tr`.
- Final states are not indicated by a final weight of `zero`.  You
  can test for finality using [`is_final`](fst_traits::CoreFst::is_final), and
  [`final_weight`](fst_traits::CoreFst::final_weight) returns an [`Option`].  This
  requires some care when converting OpenFST code.
- Transitions can be accessed directly as a slice rather than requiring
  an iterator.
- Semiring operations are expressed as plain old methods rather
  than strange C++ things.  So write `w1.plus(w2)` rather than
  `Plus(w1, w2)`, for instance.
- Weights have in-place operations for ⊕
  ([`plus_assign`](Semiring::plus_assign)) and ⊗
  ([`times_assign`](Semiring::times_assign)).
- Most of the type aliases (which would be trait aliases in Rust) such
  as `StdArc`, `StdFst`, and so forth, are missing, but type inference
  allows us to avoid explicit type arguments in most cases, such as
  when calling [`Tr::new`], for instance.
- State IDs are unsigned, with [`NO_STATE_ID`] used for a missing value.
  They are also 32 bits by default (presumably, 4 billion states
  is enough for most applications).  This means you must take care to
  cast them to [`usize`] when using them as indices, and vice-versa,
  preferably checking for overflows
- Symbol IDs are also unsigned and 32-bits, with [`NO_LABEL`] used
  for a missing value.
- Floating-point weights are not generic, so are always single-precision.

<!-- cargo-sync-readme end -->

## Benchmark with OpenFST

I did a benchmark some time ago on almost every linear fst algorithm and compared the results with `OpenFst`. You can find the results here :

- [Benchmark at the C++ level](https://github.com/garvys-org/rustfst/blob/master/bench_results/bench_funct_80.md)
- [Benchmark at the CLI level](https://github.com/garvys-org/rustfst/blob/master/bench_results/bench_cli_80.md)

Spoiler alert: `Rustfst` is faster on all those algorithms 😅

## Documentation

The documentation of the last released version is available here :
https://docs.rs/rustfst

## Release process
1. Use the script `update_version.sh` to update the version of every package.
2. Push 
3. Push a new tag with the prefix `rustfst-v`

Example :
```bash
./update_version.sh 0.9.1-alpha.6
git commit -am "Release 0.9.1-alpha.6"
git push
git tag -a rustfst-v0.9.1-alpha.6 -m "Release rustfst 0.9.1-alpha.6"  
git push --tags
```

Optionally, if this is a major release, create a GitHub release in the UI.

## Projects contained in this repository
This repository contains two main projects:
- `rustfst` is the Rust re-implementation.
  - Crate available on crates.io [here](https://crates.io/crates/rustfst)
  - Documentation available on docs.rs [here](https://docs.rs/rustfst/latest/rustfst/)
- `rustfst-python` is the python binding of `rustfst`.
  - Package available on Pypi [here](https://pypi.org/project/rustfst-python/)
  - Documentation available on Github Pages [here](https://garvys-org.github.io/rustfst/)

## License
   
Licensed under either of
- Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
