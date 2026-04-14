//! Internal stdlib-backed container shims accepting closure comparators.
//!
//! These wrap `std::collections::{BinaryHeap, BTreeMap}` to provide the
//! "comparator object" pattern previously supplied by `binary-heap-plus`
//! and `stable_bst`, without pulling in the `compare` crate.
//!
//! Single-threaded only (uses `Rc`).

#[cfg(test)]
mod tests {}
