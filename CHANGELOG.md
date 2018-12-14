# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- `write-text` function to serialize an `ExpandedFst` into text format (compatible with OpenFST).
- `draw` to generate a file representing an `ExpandedFst` in dot format that can be displayed with GraphViz.
- Implement `num_input_epsilons`, `num_output_expilons` and `relabel_pairs`.
- `closure_plus` and `closure_star` are now also available as provided methods on a `MutabeFst`.
- Add `is_one` and `is_zero` to `Semiring` API.
- Add `is_start` to `Fst` API.

### Changed


### Removed
- `determinize` no longer public as the implementation is not satisfactory.

## [0.1.7] - 2018-10-23
### Added
- First released version of rustfst

[Unreleased]: https://github.com/garvys/rustfst/compare/0.1.7...HEAD
[0.1.7]: https://github.com/garvys/rustfst/compare/0.1.2...0.1.7
