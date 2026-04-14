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

/// Stdlib-backed B-tree map whose key ordering is supplied by a closure.
///
/// Behaviourally equivalent to `stable_bst::TreeMap::with_comparator`
/// for the operations rustfst uses (`insert`, `get`, `get_or_insert`).
/// Two keys that compare `Equal` under the closure share one entry.
pub(crate) struct CmpTreeMap<K, V, F: Fn(&K, &K) -> Ordering> {
    map: std::collections::BTreeMap<KeyedKey<K, F>, V>,
    cmp: Rc<F>,
}

struct KeyedKey<K, F: Fn(&K, &K) -> Ordering> {
    key: K,
    cmp: Rc<F>,
}

impl<K, F: Fn(&K, &K) -> Ordering> PartialEq for KeyedKey<K, F> {
    fn eq(&self, other: &Self) -> bool {
        (self.cmp)(&self.key, &other.key) == Ordering::Equal
    }
}

impl<K, F: Fn(&K, &K) -> Ordering> Eq for KeyedKey<K, F> {}

impl<K, F: Fn(&K, &K) -> Ordering> Ord for KeyedKey<K, F> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.cmp)(&self.key, &other.key)
    }
}

impl<K, F: Fn(&K, &K) -> Ordering> PartialOrd for KeyedKey<K, F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Clone, V, F: Fn(&K, &K) -> Ordering> CmpTreeMap<K, V, F> {
    pub(crate) fn with_comparator(cmp: F) -> Self {
        Self {
            map: std::collections::BTreeMap::new(),
            cmp: Rc::new(cmp),
        }
    }

    pub(crate) fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.map.insert(
            KeyedKey {
                key,
                cmp: Rc::clone(&self.cmp),
            },
            value,
        )
    }

    pub(crate) fn get(&self, key: &K) -> Option<&V> {
        let probe = KeyedKey {
            key: key.clone(),
            cmp: Rc::clone(&self.cmp),
        };
        self.map.get(&probe)
    }

    pub(crate) fn get_or_insert<G: FnOnce() -> V>(&mut self, key: K, default: G) -> &mut V {
        self.map
            .entry(KeyedKey {
                key,
                cmp: Rc::clone(&self.cmp),
            })
            .or_insert_with(default)
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

    #[test]
    fn cmp_tree_map_basic_insert_get() {
        let mut map: CmpTreeMap<i32, &'static str, _> =
            CmpTreeMap::with_comparator(|a: &i32, b: &i32| a.cmp(b));
        assert_eq!(map.insert(1, "one"), None);
        assert_eq!(map.insert(2, "two"), None);
        // Re-inserting the same key returns the previous value.
        assert_eq!(map.insert(1, "uno"), Some("one"));
        assert_eq!(map.get(&1), Some(&"uno"));
        assert_eq!(map.get(&2), Some(&"two"));
        assert_eq!(map.get(&3), None);
    }

    #[test]
    fn cmp_tree_map_get_or_insert_only_runs_default_when_vacant() {
        let mut map: CmpTreeMap<i32, i32, _> =
            CmpTreeMap::with_comparator(|a: &i32, b: &i32| a.cmp(b));

        let mut counter = 0;
        // Vacant: closure runs.
        let v = map.get_or_insert(7, || {
            counter += 1;
            100
        });
        assert_eq!(*v, 100);
        assert_eq!(counter, 1);

        // Occupied: closure must NOT run.
        let v = map.get_or_insert(7, || {
            counter += 1;
            200
        });
        assert_eq!(*v, 100);
        assert_eq!(counter, 1);
    }

    #[test]
    fn cmp_tree_map_collapses_equivalence_classes() {
        // Custom comparator: two keys are "equal" when their parity matches.
        let mut map: CmpTreeMap<i32, &'static str, _> =
            CmpTreeMap::with_comparator(|a: &i32, b: &i32| (a % 2).cmp(&(b % 2)));

        map.insert(1, "odd-first");
        // 3 is "equal" to 1 under the comparator → overwrites the entry.
        assert_eq!(map.insert(3, "odd-second"), Some("odd-first"));

        // Any odd key now retrieves the same entry.
        assert_eq!(map.get(&5), Some(&"odd-second"));
        assert_eq!(map.get(&7), Some(&"odd-second"));

        // Even keys form a separate equivalence class.
        map.insert(2, "even");
        assert_eq!(map.get(&4), Some(&"even"));
        assert_eq!(map.get(&6), Some(&"even"));
    }
}
