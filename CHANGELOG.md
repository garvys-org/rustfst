# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- `write_text` function to serialize an `ExpandedFst` into text format (compatible with OpenFST).
- `draw` to generate a file representing an `ExpandedFst` in dot format that can be displayed with GraphViz.
- Implement `num_input_epsilons`, `num_output_expilons` and `relabel_pairs`.
- `closure_plus` and `closure_star` are now also available as provided methods on a `MutabeFst`.
- Add `is_one` and `is_zero` to `Semiring` API.
- Add `is_start` to `Fst` API.
- Add `text` method to write the text representation of an fst into a string.
- Add `read_text` and `from_text_string` methods to parse a text representation of an FST.
- Implement weight pushing algorithms to either start state or final states (function `push_weights`).
- Implement `reverse` operation of an FST.
- Implement `reweight` operation to modify the weights of an FST according to some potentials.
- Implement `isomorphic` operation to compare two FSTs without depending on the ordering of the states or the arcs.
- Implement a `SymbolTable` data structure to handle the mapping symbol (string) / label (int) in a FST.
- Add `symt!` macro to quickly create a `SymbolTable` from a list of strings.
- Add `fst!` macro to quickly create an acceptor or a transducer from a list of labels.
- Migrate to edition 2018 of Rust.
- Add integration tests to compare the output of this crate directly with OpenFST (by using the pynini python wrapper).
- Add weight quantization for f32 Semiring and use it in the PartialEq trait.
- Add `fst_path!` macro to easily create a FstPath object.

### Changed
- `new` method now present in the `Semiring` trait.
- In all the APIs, `StateId` are now passed by value instead of reference as it is more optimized.
- `acceptor`, `transducer`, `inverse`, `project`, `closure_plus`, `closure_star` functions can't fail anymore (no Result in API).
- Fix multiple issues when parsing an FST in text format.
- The `num_arcs` function in the `Fst` trait now computes the number of arcs leaving a specific state instead of the number of arcs in the graph.
- `acceptor` and `transducer` methods now take slices and weight instead of only iterator of labels.
- Rename `Path` object to `FstPath` to avoid conflicts with the one from the standard library.

### Removed
- `determinize` no longer public as the implementation is not satisfactory.
- In the `Semiring` trait, `one` and `zero` functions have been removed in favor of `ONE` and `ZERO` which are defined as associated constants.
- Removed `project_input` and `project_output` functions in favor of `project` with a type parameters.

## [0.1.7] - 2018-10-23
### Added
- First released version of rustfst

[Unreleased]: https://github.com/garvys/rustfst/compare/0.1.7...HEAD
[0.1.7]: https://github.com/garvys/rustfst/compare/0.1.2...0.1.7
