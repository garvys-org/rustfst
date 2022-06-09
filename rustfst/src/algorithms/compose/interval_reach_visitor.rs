use crate::algorithms::compose::{IntInterval, IntervalSet};
use crate::algorithms::dfs_visit::Visitor;
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{StateId, Tr};
use std::cmp::Ordering;

static UNASSIGNED: usize = std::usize::MAX;

pub struct IntervalReachVisitor<'a, F> {
    fst: &'a F,
    pub(crate) isets: Vec<IntervalSet>,
    pub(crate) state2index: Vec<usize>,
    index: usize,
}

impl<'a, F> IntervalReachVisitor<'a, F> {
    pub fn new(fst: &'a F) -> Self {
        Self {
            fst,
            isets: vec![],
            state2index: vec![],
            index: 1,
        }
    }
}

impl<'a, W: Semiring, F: Fst<W>> Visitor<'a, W, F> for IntervalReachVisitor<'a, F> {
    /// Invoked before DFS visit.
    fn init_visit(&mut self, _fst: &'a F) {}

    /// Invoked when state discovered (2nd arg is DFS tree root).
    fn init_state(&mut self, s: StateId, _root: StateId) -> bool {
        while self.isets.len() <= (s as usize) {
            self.isets.push(IntervalSet::default());
        }
        while self.state2index.len() <= (s as usize) {
            self.state2index.push(UNASSIGNED);
        }
        if let Some(final_weight) = self.fst.final_weight(s).unwrap() {
            if !final_weight.is_zero() {
                let interval_set = &mut self.isets[s as usize];
                if self.index == UNASSIGNED {
                    if self.fst.num_trs(s).unwrap() > 0 {
                        panic!("IntervalReachVisitor: state2index map must be empty for this FST")
                    }
                    let index = self.state2index[s as usize];
                    if index == UNASSIGNED {
                        panic!("IntervalReachVisitor: state2index map incomplete")
                    }
                    interval_set.push(IntInterval::new(index, index + 1));
                } else {
                    interval_set.push(IntInterval::new(self.index, self.index + 1));
                    self.state2index[s as usize] = self.index;
                    self.index += 1;
                }
            }
        }
        true
    }

    /// Invoked when tree transition to white/undiscovered state examined.
    fn tree_tr(&mut self, _s: StateId, _tr: &Tr<W>) -> bool {
        true
    }

    /// Invoked when back transition to grey/unfinished state examined.
    fn back_tr(&mut self, _s: StateId, _tr: &Tr<W>) -> bool {
        panic!("Cyclic input")
    }

    /// Invoked when forward or cross transition to black/finished state examined.
    fn forward_or_cross_tr(&mut self, s: StateId, tr: &Tr<W>) -> bool {
        union_vec_isets_unordered(&mut self.isets, s as usize, tr.nextstate as usize);
        true
    }

    /// Invoked when state finished ('s' is tree root, 'parent' is kNoStateId,
    /// and '_tr' is nullptr).
    fn finish_state(&mut self, s: StateId, parent: Option<StateId>, _tr: Option<&Tr<W>>) {
        if self.index != UNASSIGNED
            && self.fst.is_final(s).unwrap()
            && !self.fst.final_weight(s).unwrap().unwrap().is_zero()
        {
            let intervals = &mut self.isets[s as usize].intervals.intervals;
            intervals[0].end = self.index;
        }
        self.isets[s as usize].normalize();
        if let Some(p) = parent {
            union_vec_isets_unordered(&mut self.isets, p as usize, s as usize);
        }
    }

    /// Invoked after DFS visit.
    fn finish_visit(&mut self) {}
}

// Perform the union of two IntervalSet stored in a vec. Utils to fix issue with borrow checker.
fn union_vec_isets_unordered(isets: &mut [IntervalSet], i: usize, j: usize) {
    debug_assert_ne!(i, j);
    match i.cmp(&j) {
        Ordering::Less => {
            let (v_0_isupm1, v_isup1_end) = isets.split_at_mut(j);
            v_0_isupm1[i].union(v_isup1_end[0].clone());
        }
        Ordering::Greater => {
            let (v_0_jsupm1, v_jsup1_end) = isets.split_at_mut(i);
            v_jsup1_end[0].union(v_0_jsupm1[j].clone());
        }
        Ordering::Equal => {
            panic!("Unreachable code")
        }
    }
}
