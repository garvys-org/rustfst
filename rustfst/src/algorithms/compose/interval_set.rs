use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::slice::Iter as IterSlice;
use std::vec::IntoIter as IntoIterVec;
use superslice::Ext;
use unsafe_unwrap::UnsafeUnwrap;

/// Half-open integral interval [a, b) of signed integers of type T.
#[derive(PartialEq, Clone, Eq, Debug, Serialize, Deserialize)]
pub struct IntInterval {
    pub begin: usize,
    pub end: usize,
}

impl IntInterval {
    pub fn new(begin: usize, end: usize) -> Self {
        Self { begin, end }
    }
}

// Not using default implementation to make sure that begin is compared first
// as it plays a role in the normalize function
impl PartialOrd for IntInterval {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IntInterval {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.begin.cmp(&other.begin) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => {
                // self.begin == other.begin
                match self.end.cmp(&other.end) {
                    Ordering::Less => Ordering::Greater,
                    Ordering::Greater => Ordering::Less,
                    Ordering::Equal => Ordering::Equal,
                }
            }
        }
    }
}

/// Stores IntIntervals in a vector. In addition, keeps the count of points in
/// all intervals.
#[derive(Clone, PartialOrd, PartialEq, Debug, Default)]
pub struct VectorIntervalStore {
    pub(crate) intervals: Vec<IntInterval>,
    count: Option<usize>,
}

impl VectorIntervalStore {
    pub fn len(&self) -> usize {
        self.intervals.len()
    }

    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }

    pub fn push(&mut self, interval: IntInterval) {
        self.intervals.push(interval)
    }

    pub fn clear(&mut self) {
        self.intervals.clear();
        self.count = None;
    }

    pub fn count(&self) -> Option<usize> {
        self.count
    }

    pub fn set_count(&mut self, count: usize) {
        self.count = Some(count);
    }

    pub fn iter(&self) -> IterSlice<IntInterval> {
        self.intervals.iter()
    }

    pub fn into_iter(self) -> IntoIterVec<IntInterval> {
        self.intervals.into_iter()
    }
}

#[derive(PartialOrd, PartialEq, Default, Clone, Debug)]
pub struct IntervalSet {
    pub(crate) intervals: VectorIntervalStore,
}

impl IntoIterator for IntervalSet {
    type Item = IntInterval;
    type IntoIter = IntoIterVec<IntInterval>;

    fn into_iter(self) -> Self::IntoIter {
        self.intervals.into_iter()
    }
}

impl IntervalSet {
    pub fn len(&self) -> usize {
        self.intervals.len()
    }

    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }

    pub fn push(&mut self, interval: IntInterval) {
        self.intervals.push(interval)
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
    pub fn union(&mut self, iset: Self) {
        self.intervals.intervals.extend(iset)
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
                // Empty interval
                continue;
            }
            for j in (inti_index + 1)..n_intervals {
                let intj = unsafe { intervals_ip1_end.get_unchecked_mut(j - (inti_index + 1)) };
                if intj.begin > inti.end {
                    // No overlap between the two intervals
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
        self.intervals.set_count(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_normalize_interval_set() -> Result<()> {
        let mut interval_set = IntervalSet::default();

        assert!(!interval_set.singleton());
        assert!(!interval_set.member(3));

        interval_set.push(IntInterval::new(0, 5));

        interval_set.push(IntInterval::new(3, 10));

        interval_set.normalize();

        assert!(interval_set.member(3));

        {
            let mut ref_interval_set = IntervalSet::default();
            ref_interval_set.push(IntInterval::new(0, 10));
            ref_interval_set.intervals.set_count(10);

            assert_eq!(interval_set, ref_interval_set);
        }

        let mut interval_set_2 = IntervalSet::default();
        interval_set_2.push(IntInterval::new(12, 13));
        assert!(interval_set_2.singleton());

        interval_set.union(interval_set_2);
        interval_set.normalize();

        {
            let mut ref_interval_set = IntervalSet::default();
            ref_interval_set.push(IntInterval::new(0, 10));
            ref_interval_set.push(IntInterval::new(12, 13));
            ref_interval_set.intervals.set_count(11);

            assert_eq!(interval_set, ref_interval_set);
        }

        Ok(())
    }

    #[test]
    fn test_ord_intinterval() -> Result<()> {
        {
            let interval_1 = IntInterval::new(1, 4);
            let interval_2 = IntInterval::new(2, 3);
            assert_eq!(interval_1.cmp(&interval_2), Ordering::Less);
        }

        {
            let interval_1 = IntInterval::new(1, 4);
            let interval_2 = IntInterval::new(1, 4);
            assert_eq!(interval_1.cmp(&interval_2), Ordering::Equal);
        }

        {
            let interval_1 = IntInterval::new(3, 4);
            let interval_2 = IntInterval::new(2, 3);
            assert_eq!(interval_1.cmp(&interval_2), Ordering::Greater);
        }

        {
            let interval_1 = IntInterval::new(1, 4);
            let interval_2 = IntInterval::new(1, 3);
            assert_eq!(interval_1.cmp(&interval_2), Ordering::Less);
        }

        {
            let interval_1 = IntInterval::new(1, 4);
            let interval_2 = IntInterval::new(1, 5);
            assert_eq!(interval_1.cmp(&interval_2), Ordering::Greater);
        }

        Ok(())
    }
}
