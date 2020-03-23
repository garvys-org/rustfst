use num_traits::PrimInt;
use std::collections::HashSet;
use std::slice::Iter as IterSlice;
use superslice::Ext;
use unsafe_unwrap::UnsafeUnwrap;

/// Half-open integral interval [a, b) of signed integers of type T.
#[derive(PartialOrd, PartialEq, Clone, Ord, Eq)]
pub struct IntInterval {
    pub(crate) begin: usize,
    pub(crate) end: usize,
}

impl IntInterval {
    pub fn new(begin: usize, end: usize) -> Self {
        Self { begin, end }
    }
}

/// Stores IntIntervals<T> in a vector. In addition, keeps the count of points in
/// all intervals.
#[derive(Clone, PartialOrd, PartialEq)]
pub struct VectorIntervalStore {
    pub(crate) intervals: Vec<IntInterval>,
    count: Option<usize>,
}

impl Default for VectorIntervalStore {
    fn default() -> Self {
        Self {
            intervals: Vec::new(),
            count: None,
        }
    }
}

impl VectorIntervalStore {
    pub fn len(&self) -> usize {
        self.intervals.len()
    }

    pub fn clear(&mut self) {
        self.intervals.clear();
        self.count = None;
    }

    pub fn count(&self) -> Option<usize> {
        self.count.clone()
    }

    pub fn set_count(&mut self, count: usize) {
        self.count = Some(count);
    }

    pub fn iter(&self) -> IterSlice<IntInterval> {
        self.intervals.iter()
    }
}

#[derive(PartialOrd, PartialEq, Default, Clone)]
pub struct IntervalSet {
    pub(crate) intervals: VectorIntervalStore,
}

impl IntervalSet {
    pub fn len(&self) -> usize {
        self.intervals.len()
    }

    // Number of points in the intervals (undefined if not normalized).
    pub fn count(&self) -> Option<usize> {
        self.intervals.count()
    }

    pub fn clear(&mut self) {
        self.intervals.clear()
    }

    pub fn iter(&self) -> IterSlice<IntInterval> {
        self.intervals.iter()
    }

    // Adds an interval set to the set. The result may not be normalized.
    pub fn union(&mut self, iset: &Self) {
        self.intervals
            .intervals
            .extend(iset.intervals.iter().cloned())
    }

    // Requires intervals be normalized.
    pub fn member(&self, value: usize) -> bool {
        let interval = IntInterval::new(value, value);
        let lb = self.intervals.intervals.lower_bound(&interval);
        if lb == 0 {
            return false;
        }
        self.intervals.intervals[lb - 1].end > value
    }

    pub fn singleton(&self) -> bool {
        if self.len() != 1 {
            return false;
        }
        let elt = unsafe { self.intervals.iter().next().unsafe_unwrap() };
        elt.begin + 1 == elt.end
    }

    // Sorts, collapses overlapping and adjacent interals, and sets count.
    pub fn normalize(&mut self) {
        let intervals = &mut self.intervals.intervals;
        intervals.sort();
        let n_intervals = intervals.len();
        let mut count = 0;
        let mut intervals_indexes_to_keep = HashSet::new();
        let mut i = 0;
        while i < n_intervals {
            let (intervals_0_i, intervals_ip1_end) = intervals.split_at_mut(i + 1);
            let inti = unsafe { intervals_0_i.get_unchecked_mut(i) };
            let inti_index = i;
            if inti.begin == inti.end {
                continue;
            }
            for j in i + 1..n_intervals {
                let intj = unsafe { intervals_ip1_end.get_unchecked_mut(j - (i + 1)) };
                if intj.begin > inti.end {
                    break;
                }
                if intj.end > inti.end {
                    inti.end = intj.end;
                }
                i += 1;
            }
            count += inti.end - inti.begin;
            intervals_indexes_to_keep.insert(inti_index);

            // Loop incrementation
            i += 1;
        }

        let mut index = 0;
        self.intervals
            .intervals
            .retain(|_| (intervals_indexes_to_keep.contains(&index), index += 1).0);
    }
}
