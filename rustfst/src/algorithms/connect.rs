use std::collections::HashSet;
use std::hash::BuildHasher;

use failure::Fallible;
use unsafe_unwrap::UnsafeUnwrap;

use crate::{Arc, NO_STATE_ID};
use crate::fst_traits::{ExpandedFst, MutableFst, CoreFst};
use crate::fst_traits::ArcIterator;
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::StateId;
use crate::algorithms::cache::VectorCacheState;

/// This operation trims an FST, removing states and arcs that are not on successful paths.
///
/// # Example
/// ```
/// # #[macro_use] extern crate rustfst;
/// # use rustfst::utils::transducer;
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::algorithms::connect;
/// # use rustfst::fst_traits::MutableFst;
/// let fst : VectorFst<IntegerWeight> = fst![2 => 3];
///
/// // Add a state not on a successful path
/// let mut no_connected_fst = fst.clone();
/// no_connected_fst.add_state();
///
/// let mut connected_fst = no_connected_fst.clone();
/// connect(&mut connected_fst);
///
/// assert_eq!(connected_fst, fst);
/// ```
pub fn connect<F: ExpandedFst + MutableFst>(fst: &mut F) -> Fallible<()> {
    let mut visitor = SccVisitor::new(fst);
    dfs_visit(fst, &mut visitor, false);
    let mut dstates = Vec::with_capacity(visitor.access.len());
    for s in 0..visitor.access.len() {
        if !visitor.access[s] || !visitor.coaccess[s] {
            dstates.push(s);
        }
    }
    fst.del_states(dstates)?;
    Ok(())
}

#[derive(PartialOrd, PartialEq, Copy, Clone)]
enum StateColor {
    DfsWhite, // Undiscovered.
    DfsGrey,  // Discovered but unfinished.
    DfsBlack, // Finished.
}

pub trait Visitor<'a, F: Fst> {
    /// Invoked before DFS visit.
    fn init_visit(&mut self, fst: &'a F);

    /// Invoked when state discovered (2nd arg is DFS tree root).
    fn init_state(&mut self, s: StateId, root: StateId) -> bool;

    /// Invoked when tree arc to white/undiscovered state examined.
    fn tree_arc(&mut self, s: StateId, arc: &Arc<F::W>) -> bool;

    /// Invoked when back arc to grey/unfinished state examined.
    fn back_arc(&mut self, s: StateId, arc: &Arc<F::W>) -> bool;

    /// Invoked when forward or cross arc to black/finished state examined.
    fn forward_or_cross_arc(&mut self, s: StateId, arc: &Arc<F::W>) -> bool;

    /// Invoked when state finished ('s' is tree root, 'parent' is kNoStateId,
    /// and 'arc' is nullptr).
    fn finish_state(&mut self, s: StateId, parent: Option<StateId>, arc: Option<&Arc<F::W>>);

    /// Invoked after DFS visit.
    fn finish_visit(&mut self);
}

struct SccVisitor<'a, F: Fst> {
    scc: Vec<i32>,
    access: Vec<bool>,
    coaccess: Vec<bool>,
    start: i32,
    fst: &'a F,
    nstates : usize,
    nscc: usize,
    dfnumber: Vec<i32>,
    lowlink: Vec<i32>,
    onstack: Vec<bool>,
    scc_stack: Vec<StateId>
}

impl<'a, F: 'a + Fst>  SccVisitor<'a, F> {
    pub fn new(fst: &'a F) -> Self {
        Self {
            scc: vec![],
            access: vec![],
            coaccess: vec![],
            start: fst.start().map(|v| v as i32).unwrap_or(-1),
            fst,
            nstates: 0,
            nscc: 0,
            dfnumber: vec![],
            lowlink: vec![],
            onstack: vec![],
            scc_stack: vec![]
        }
    }
}

impl<'a, F: 'a + Fst> Visitor<'a, F> for SccVisitor<'a, F> {
    fn init_visit(&mut self, fst: &'a F) {
        self.scc.clear();
        self.access.clear();
        self.coaccess.clear();
        self.start = fst.start().map(|v| v as i32).unwrap_or(-1);
        self.fst = fst;
        self.nstates = 0;
        self.nscc = 0;
        self.dfnumber.clear();
        self.lowlink.clear();
        self.onstack.clear();
        self.scc_stack.clear();
    }

    fn init_state(&mut self, s: usize, root: usize) -> bool {
        self.scc_stack.push(s);
        if self.dfnumber.len() <= s {
            self.scc.resize(s+1, -1);
            self.access.resize(s+1, false);
            self.coaccess.resize(s+1, false);
            self.dfnumber.resize(s+1, -1);
            self.lowlink.resize(s+1, -1);
            self.onstack.resize(s+1, false);
        }
        self.dfnumber[s] = self.nstates as i32;
        self.lowlink[s] = self.nstates as i32;
        self.onstack[s] = true;
        if root as i32 == self.start {
            self.access[s] = true;
        } else {
            self.access[s] = false;
        }
        self.nstates += 1;
        true
    }

    fn tree_arc(&mut self, s: usize, arc: &Arc<<F as CoreFst>::W>) -> bool {
        true
    }

    fn back_arc(&mut self, s: usize, arc: &Arc<<F as CoreFst>::W>) -> bool {
        let t = arc.nextstate;
        if self.dfnumber[t] < self.lowlink[s] {
            self.lowlink[s] = self.dfnumber[t];
        }
        if self.coaccess[t] {
            self.coaccess[s] = true;
        }
        true
    }

    fn forward_or_cross_arc(&mut self, s: usize, arc: &Arc<<F as CoreFst>::W>) -> bool {
        let t = arc.nextstate;
        if self.dfnumber[t] < self.dfnumber[s] && self.onstack[t] && self.dfnumber[t] < self.lowlink[s] {
            self.lowlink[s] = self.dfnumber[t];
        }
        if self.coaccess[t] {
            self.coaccess[s] = true;
        }
        true
    }

    fn finish_state(&mut self, s: usize, parent: Option<usize>, arc: Option<&Arc<<F as CoreFst>::W>>) {
        if self.fst.is_final(s) {
            self.coaccess[s] = true;
        }
        if self.dfnumber[s] == self.lowlink[s] {
            let mut scc_coaccess = false;
            let mut i = self.scc_stack.len();
            let mut t = 0;
            loop {
                i -= 1;
                t = self.scc_stack[i];
                if self.coaccess[t] {
                    scc_coaccess = true;
                }
                if s == t {
                    break
                }
            }
            loop {
                t = *self.scc_stack.last().unwrap();
                self.scc[t] = self.nscc as i32;
                if scc_coaccess {
                    self.coaccess[t] = true;
                }
                self.onstack[t] = false;
                self.scc_stack.pop();
                if s == t {
                    break
                }
            }
            self.nscc += 1;
        }
        if let Some(_p) = parent {
            if self.coaccess[s] {
                self.coaccess[_p] = true;
            }
            if self.lowlink[s] < self.lowlink[_p] {
                self.lowlink[_p] = self.lowlink[s];
            }
        }
    }

    fn finish_visit(&mut self) {
        for s in 0..self.scc.len() {
            self.scc[s] = self.nscc as i32 - 1 - self.scc[s];
        }
        self.dfnumber.clear();
        self.lowlink.clear();
        self.onstack.clear();
        self.scc_stack.clear();
    }
}

struct DfsState<'a, W, AI>
where
    W: Semiring + 'a,
    AI: Iterator<Item = &'a Arc<W>> + Clone,
{
    state_id: StateId,
    arc_iter: OpenFstIterator<AI>,
}

impl<'a, W, AI> DfsState<'a, W, AI>
where
    W: Semiring + 'a,
    AI: Iterator<Item = &'a Arc<W>> + Clone,
{
    pub fn new<F: ArcIterator<'a, Iter = AI, W = W>>(fst: &'a F, s: StateId) -> Self {
        Self {
            state_id: s,
            arc_iter: OpenFstIterator::new(fst.arcs_iter_unchecked(s)),
        }
    }
}

struct OpenFstIterator<I: Iterator> {
    iter: I,
    e: Option<I::Item>,
}

impl<I: Iterator> OpenFstIterator<I> {
    pub fn new(mut iter: I) -> Self {
        let e = iter.next();
        Self { iter, e }
    }

    pub fn value(&self) -> &I::Item {
        unsafe { self.e.as_ref().unsafe_unwrap() }
    }

    pub fn done(&self) -> bool {
        self.e.is_none()
    }

    pub fn next(&mut self) {
        self.e = self.iter.next();
    }
}

pub fn dfs_visit<'a, F: Fst + ExpandedFst, V: Visitor<'a, F>>(fst: &'a F, visitor: &mut V, access_only: bool) {
    visitor.init_visit(fst);
    let start = fst.start();
    if start.is_none() {
        visitor.finish_visit();
        return;
    }
    let start = unsafe { start.unsafe_unwrap() };

    let nstates = fst.num_states();
    let mut state_color = vec![StateColor::DfsWhite; nstates];
    let mut state_stack = vec![];

    // Continue dfs while true.
    let mut dfs = true;
    let mut root = start;
    loop {
        if !dfs || root >= nstates {
            break;
        }
        state_color[root] = StateColor::DfsGrey;
        state_stack.push(DfsState::new(fst, root));
        dfs = visitor.init_state(root, root);
                let mut state_stack_next = None;
        while !state_stack.is_empty() {
            let dfs_state = unsafe { state_stack.last_mut().unsafe_unwrap() };
            let s = dfs_state.state_id;
            let aiter = &mut dfs_state.arc_iter;
            if !dfs || aiter.done() {
                state_color[s] = StateColor::DfsBlack;
                state_stack.pop();
                if !state_stack.is_empty() {
                    let parent_state = unsafe { state_stack.last_mut().unsafe_unwrap() };
                    let piter = &mut parent_state.arc_iter;
                    visitor.finish_state(s, Some(parent_state.state_id), Some(*piter.value()));
                    piter.next();
                } else {
                    visitor.finish_state(s, None, None);
                }
                continue;
            }
            let arc = aiter.value();
            let next_color = state_color[arc.nextstate];
            match next_color {
                StateColor::DfsWhite => {
                    dfs = visitor.tree_arc(s, arc);
                    if !dfs {
                        break;
                    }
                    state_color[arc.nextstate] = StateColor::DfsGrey;
                    state_stack_next = Some(DfsState::new(fst, arc.nextstate));
                    dfs = visitor.init_state(arc.nextstate, root);
                }
                StateColor::DfsGrey => {
                    dfs = visitor.back_arc(s, arc);
                    aiter.next();
                }
                StateColor::DfsBlack => {
                    dfs = visitor.forward_or_cross_arc(s, arc);
                    aiter.next();
                }
            };

            // Fix issues with borrow checker.
            if let Some(a) = state_stack_next.take() {
                state_stack.push(a);
            }
        }

        if access_only {break;}

        root = if root == start {
            0
        } else {
            root + 1
        };

        while root < nstates && state_color[root] != StateColor::DfsWhite {
            root += 1;
        }
    }
    visitor.finish_visit();
}

#[cfg(test)]
mod tests {
    use crate::test_data::vector_fst::get_vector_fsts_for_tests;

    use super::*;

    #[test]
    fn test_connect_generic() -> Fallible<()> {
        for data in get_vector_fsts_for_tests() {
            let fst = &data.fst;

            let mut connect_fst = fst.clone();
            connect(&mut connect_fst)?;

            assert_eq!(
                connect_fst, data.connected_fst,
                "Connect test fail for fst : {:?}",
                &data.name
            );
        }
        Ok(())
    }
}
