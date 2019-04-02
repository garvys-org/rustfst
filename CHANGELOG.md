# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Add `arc_map` function to perform modifications that don't modify the number of arcs (and `ArcMapper` to specify how to modify the arcs).
- Implemented the following arc mappers to be used with `arc_map`:
   - `IdentityMapper`: Mapper that returns its input.
   - `InputEpsilonMapper`: Mapper that converts all input symbols to epsilon.
   - `InvertWeightMapper`: Mapper to inverse all non-Zero() weights.
   - `OutputEpsilonMapper`: Mapper that converts all output symbols to epsilon.
   - `PlusMapper`: Mapper to add a constant to all weights.
   - `QuantizeMapper`: Mapper to quantize all weights.
   - `RmWeightMapper`: Mapper to map all non-Zero() weights to One().
   - `TimesMapper`: Mapper to (right) multiply a constant to all weights.
- Add `final_weight_mut` to the `MutableFst` trait to retrieve a mutable reference on the final weight and implemented for `VectorFst`.
- `arc_map` method has been added to the MutableFst trait as a provided method.
- Add `num-traits` and `ordered-float` as dependency because hashing float is needed (yeah this is bad).
- Add struct `FinalArc` to correctly handle final weight when performing arc mapping.
- Add enum `MapFinalAction` to handle the creation of super final states when performing arc mapping.
- Implement `encode` and `decode` functions to allow the representation of a weighted transducer as a weighted automaton, an unweighted transducer or an unweighted automaton.
- Implement `rm_final_epsilon` which removes final states that have epsilon-only input arcs.
- Add `delete_final_weight` and `delete_arcs` to the MutableFst trait and implement them for `VectorFst`.
- Add `ArcFilter` trait and implement it for `AnyArcFilter`, `EpsilonArcFilter`, `InputEpsilonArcFilter` and `OutputEpsilonArcFilter`.
- Add `state_map` function to perform state modifications and `StateMapper` to specify the modifications.
- Implemented the following state mappers :
    - `ArcSumMapper`: Mapper that Plus-sum the arcs with same nextstate, ilabel and olabel.
    - `ArcUniqueMapper`: Remove duplicate arcs.
    - `IdentityStateMapper`: Return its input.
- Add `pop_arcs`, `reserve_arcs` and `reserve_state` to the `MutableFst` API.
- `state_map` method has been added to the MutableFst trait as a provided method.
- Implement `weight_convert` and `WeightConverter` to changed the Semiring of an FST.
- Implement WeightConverter trait for `IdentityArcMapper`, `InputEpsilonMapper`, `InvertWeightMapper`, `OutputEpsilonMapper`, `PlusMapper`, `QuantizeMapper`, `RmWeightMapper`, `TimesMapper`, 
- Add `arcsort` to sort the outgoing arcs of an FST.
- Add `ilabel_compare`, `olabel_compare` and `arc_compare` to compare two arcs. Can be used with `arcsort`.
- Add `GallicWeight` and `StringWeight`.
- Add `ProductWeight` : Weight W1 x W2
- Add `PowerWeight`: Weoght W ^ N
- Add `UnionWeight`
- Add `state_map` and `StateMapper`. Implement `StateMapper` for `ArcSumMapper` and `AddUniqueMapper`.
- Implement `WeightConverter` for `FromGallicMapper` and `ToGallicMapper`.
- Add `is_acceptor` to the FST trait to detect whether an FST is an acceptor.
- Implement `determinize` for both acceptors and transducers.

### Changed
- In the `Semiring` trait, `ONE` and `ZERO` associated constants have been removed in favor of `one` and `zero` functions.
- In the Semiring, `plus` and `times` methods now accept a `AsrRef<Self>` parameter.
- Added `inverse_assign`, `quantize_assign`, `plus_assign` and `times_assign` to the Semiring trait to perform in place operations.
- Changed `ArcMapper` trait to use `FinalArc` and `MapFinalAction`.
- Implement `invert` and `project` using `arc_map`.
- Change internal representation of float weights to use the `ordered-float` crate and derive `Hash` and `Eq` for each Semiring.
- `ArcSumMapper` is now called inside `rm_epsilon` function to conform with OpenFST.
- `plus` and `times` in Semiring now returns Fallible.
- `ArcMapper` returns Fallible.

### Removed
- Removed `Add`, `AddAssign`, `Mul`, `MulAssign` and `Copy` trait bound for `Semiring`.
- Removed custom `Result` type and replace it with `Fallible` from failure. Underneath, it is the same type.

## [0.2.0] - 2019-01-07
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

[Unreleased]: https://github.com/garvys/rustfst/compare/0.2.0...HEAD
[0.2.0]: https://github.com/garvys/rustfst/compare/0.1.7...0.2.0
[0.1.7]: https://github.com/garvys/rustfst/compare/0.1.2...0.1.7
