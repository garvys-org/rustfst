//! Internal stdlib-backed container shims accepting closure comparators.
//!
//! These wrap `std::collections::{BinaryHeap, BTreeMap}` to provide the
//! "comparator object" pattern previously supplied by `binary-heap-plus`
//! and `stable_bst`, without pulling in the `compare` crate.
//!
//! Single-threaded only (uses `Rc`).

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt;
use std::rc::Rc;

/// Stdlib-backed binary heap whose ordering is supplied by a closure.
///
/// Behaviourally equivalent to `binary_heap_plus::BinaryHeap::new_by`:
/// the closure plays the same role as `Ord::cmp`, so the *largest*
/// element per the closure pops first.
pub(crate) struct CmpHeap<T, F: Fn(&T, &T) -> Ordering> {
    heap: BinaryHeap<Item<T, F>>,
    cmp: Rc<F>,
}

struct Item<T, F: Fn(&T, &T) -> Ordering> {
    val: T,
    cmp: Rc<F>,
}

impl<T, F: Fn(&T, &T) -> Ordering> PartialEq for Item<T, F> {
    fn eq(&self, other: &Self) -> bool {
        (self.cmp)(&self.val, &other.val) == Ordering::Equal
    }
}

impl<T, F: Fn(&T, &T) -> Ordering> Eq for Item<T, F> {}

impl<T, F: Fn(&T, &T) -> Ordering> Ord for Item<T, F> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.cmp)(&self.val, &other.val)
    }
}

impl<T, F: Fn(&T, &T) -> Ordering> PartialOrd for Item<T, F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T, F: Fn(&T, &T) -> Ordering> CmpHeap<T, F> {
    pub(crate) fn new_by(cmp: F) -> Self {
        Self {
            heap: BinaryHeap::new(),
            cmp: Rc::new(cmp),
        }
    }

    pub(crate) fn push(&mut self, val: T) {
        self.heap.push(Item {
            val,
            cmp: Rc::clone(&self.cmp),
        });
    }

    pub(crate) fn pop(&mut self) -> Option<T> {
        self.heap.pop().map(|i| i.val)
    }

    pub(crate) fn peek(&self) -> Option<&T> {
        self.heap.peek().map(|i| &i.val)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub(crate) fn len(&self) -> usize {
        self.heap.len()
    }

    pub(crate) fn clear(&mut self) {
        self.heap.clear();
    }
}

impl<T: Clone, F: Fn(&T, &T) -> Ordering + Clone> Clone for CmpHeap<T, F> {
    fn clone(&self) -> Self {
        let cmp = Rc::new((*self.cmp).clone());
        let heap = self
            .heap
            .iter()
            .map(|i| Item {
                val: i.val.clone(),
                cmp: Rc::clone(&cmp),
            })
            .collect();
        Self { heap, cmp }
    }
}

impl<T: fmt::Debug, F: Fn(&T, &T) -> Ordering> fmt::Debug for CmpHeap<T, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.heap.iter().map(|i| &i.val))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmp_heap_min_heap_via_closure() {
        // Min-heap over i32: closure says `a` is "greater" when `a` is
        // numerically smaller. Stdlib BinaryHeap pops the max-by-Ord
        // first, so the smallest pops first.
        let mut heap: CmpHeap<i32, _> = CmpHeap::new_by(|a: &i32, b: &i32| b.cmp(a));
        for v in [3, 1, 4, 1, 5, 9, 2, 6] {
            heap.push(v);
        }
        assert_eq!(heap.len(), 8);
        assert!(!heap.is_empty());
        assert_eq!(heap.peek(), Some(&1));

        let mut popped = Vec::new();
        while let Some(v) = heap.pop() {
            popped.push(v);
        }
        assert_eq!(popped, vec![1, 1, 2, 3, 4, 5, 6, 9]);
        assert!(heap.is_empty());
        assert_eq!(heap.len(), 0);
    }

    #[test]
    fn cmp_heap_clear_resets() {
        let mut heap: CmpHeap<i32, _> = CmpHeap::new_by(|a: &i32, b: &i32| a.cmp(b));
        heap.push(10);
        heap.push(20);
        heap.clear();
        assert!(heap.is_empty());
        assert_eq!(heap.len(), 0);
        assert_eq!(heap.pop(), None);
    }

    #[test]
    fn cmp_heap_clone_produces_equivalent_copy() {
        let mut original: CmpHeap<i32, _> = CmpHeap::new_by(|a: &i32, b: &i32| b.cmp(a));
        for v in [3, 1, 4, 1, 5] {
            original.push(v);
        }
        let mut cloned = original.clone();

        // Mutating the clone must not affect the original.
        cloned.pop();
        assert_eq!(original.len(), 5);
        assert_eq!(cloned.len(), 4);

        // Both heaps should still drain in min-order.
        let mut from_orig = Vec::new();
        while let Some(v) = original.pop() {
            from_orig.push(v);
        }
        let mut from_clone = Vec::new();
        while let Some(v) = cloned.pop() {
            from_clone.push(v);
        }
        assert_eq!(from_orig, vec![1, 1, 3, 4, 5]);
        assert_eq!(from_clone, vec![1, 3, 4, 5]);
    }
}
