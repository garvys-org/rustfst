use crate::arc::Arc;
use crate::fst_traits::{ArcIterator, ExpandedFst, Fst};
use crate::semirings::Semiring;
use crate::StateId;

use crate::algorithms::arc_filters::ArcFilter;
use unsafe_unwrap::UnsafeUnwrap;

#[derive(PartialOrd, PartialEq, Copy, Clone)]
enum DfsStateColor {
    /// Undiscovered.
    White,
    /// Discovered but unfinished.
    Grey,
    /// Finished.
    Black,
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
    #[inline]
    pub fn new<F: ArcIterator<'a, Iter = AI, W = W>>(fst: &'a F, s: StateId) -> Self {
        Self {
            state_id: s,
            arc_iter: OpenFstIterator::new(unsafe { fst.arcs_iter_unchecked(s) }),
        }
    }
}

struct OpenFstIterator<I: Iterator> {
    iter: I,
    e: Option<I::Item>,
}

impl<I: Iterator> OpenFstIterator<I> {
    #[inline]
    fn new(mut iter: I) -> Self {
        let e = iter.next();
        Self { iter, e }
    }

    #[inline]
    fn value(&self) -> &I::Item {
        unsafe { self.e.as_ref().unsafe_unwrap() }
    }

    #[inline]
    fn done(&self) -> bool {
        self.e.is_none()
    }

    #[inline]
    fn next(&mut self) {
        self.e = self.iter.next();
    }
}

pub fn dfs_visit<'a, F: Fst + ExpandedFst, V: Visitor<'a, F>, A: ArcFilter<F::W>>(
    fst: &'a F,
    visitor: &mut V,
    arc_filter: &A,
    access_only: bool,
) {
    visitor.init_visit(fst);
    let start = fst.start();
    if start.is_none() {
        visitor.finish_visit();
        return;
    }
    let start = unsafe { start.unsafe_unwrap() };

    let nstates = fst.num_states();
    let mut state_color = vec![DfsStateColor::White; nstates];
    let mut state_stack = vec![];

    // Continue dfs while true.
    let mut dfs = true;
    let mut root = start;
    loop {
        if !dfs || root >= nstates {
            break;
        }
        state_color[root] = DfsStateColor::Grey;
        state_stack.push(DfsState::new(fst, root));
        dfs = visitor.init_state(root, root);
        let mut state_stack_next = None;
        while !state_stack.is_empty() {
            let dfs_state = unsafe { state_stack.last_mut().unsafe_unwrap() };
            let s = dfs_state.state_id;
            let aiter = &mut dfs_state.arc_iter;
            if !dfs || aiter.done() {
                state_color[s] = DfsStateColor::Black;
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
            if !(arc_filter.keep(arc)) {
                aiter.next();
                continue;
            }
            match next_color {
                DfsStateColor::White => {
                    dfs = visitor.tree_arc(s, arc);
                    if !dfs {
                        break;
                    }
                    state_color[arc.nextstate] = DfsStateColor::Grey;
                    state_stack_next = Some(DfsState::new(fst, arc.nextstate));
                    dfs = visitor.init_state(arc.nextstate, root);
                }
                DfsStateColor::Grey => {
                    dfs = visitor.back_arc(s, arc);
                    aiter.next();
                }
                DfsStateColor::Black => {
                    dfs = visitor.forward_or_cross_arc(s, arc);
                    aiter.next();
                }
            };

            // Fix issues with borrow checker.
            if let Some(a) = state_stack_next.take() {
                state_stack.push(a);
            }
        }

        if access_only {
            break;
        }

        root = if root == start { 0 } else { root + 1 };

        while root < nstates && state_color[root] != DfsStateColor::White {
            root += 1;
        }
    }
    visitor.finish_visit();
}
