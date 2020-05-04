use crate::fst_traits::{ExpandedFst, Fst, TrIterator};
use crate::semirings::Semiring;
use crate::tr::Tr;
use crate::StateId;

use crate::algorithms::tr_filters::TrFilter;
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

    /// Invoked when tree tr to white/undiscovered state examined.
    fn tree_tr(&mut self, s: StateId, tr: &Tr<F::W>) -> bool;

    /// Invoked when back tr to grey/unfinished state examined.
    fn back_tr(&mut self, s: StateId, tr: &Tr<F::W>) -> bool;

    /// Invoked when forward or cross tr to black/finished state examined.
    fn forward_or_cross_tr(&mut self, s: StateId, tr: &Tr<F::W>) -> bool;

    /// Invoked when state finished ('s' is tree root, 'parent' is kNoStateId,
    /// and 'tr' is nullptr).
    fn finish_state(&mut self, s: StateId, parent: Option<StateId>, tr: Option<&Tr<F::W>>);

    /// Invoked after DFS visit.
    fn finish_visit(&mut self);
}

struct DfsState<'a, W, AI>
where
    W: Semiring + 'a,
    AI: Iterator<Item = &'a Tr<W>> + Clone,
{
    state_id: StateId,
    tr_iter: OpenFstIterator<AI>,
}

impl<'a, W, AI> DfsState<'a, W, AI>
where
    W: Semiring + 'a,
    AI: Iterator<Item = &'a Tr<W>> + Clone,
{
    #[inline]
    pub fn new<F: TrIterator<'a, Iter = AI, W = W>>(fst: &'a F, s: StateId) -> Self {
        Self {
            state_id: s,
            tr_iter: OpenFstIterator::new(unsafe { fst.tr_iter_unchecked(s) }),
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

pub fn dfs_visit<'a, F: Fst + ExpandedFst, V: Visitor<'a, F>, A: TrFilter<F::W>>(
    fst: &'a F,
    visitor: &mut V,
    tr_filter: &A,
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
            let aiter = &mut dfs_state.tr_iter;
            if !dfs || aiter.done() {
                state_color[s] = DfsStateColor::Black;
                state_stack.pop();
                if !state_stack.is_empty() {
                    let parent_state = unsafe { state_stack.last_mut().unsafe_unwrap() };
                    let piter = &mut parent_state.tr_iter;
                    visitor.finish_state(s, Some(parent_state.state_id), Some(*piter.value()));
                    piter.next();
                } else {
                    visitor.finish_state(s, None, None);
                }
                continue;
            }
            let tr = aiter.value();
            let next_color = state_color[tr.nextstate];
            if !(tr_filter.keep(tr)) {
                aiter.next();
                continue;
            }
            match next_color {
                DfsStateColor::White => {
                    dfs = visitor.tree_tr(s, tr);
                    if !dfs {
                        break;
                    }
                    state_color[tr.nextstate] = DfsStateColor::Grey;
                    state_stack_next = Some(DfsState::new(fst, tr.nextstate));
                    dfs = visitor.init_state(tr.nextstate, root);
                }
                DfsStateColor::Grey => {
                    dfs = visitor.back_tr(s, tr);
                    aiter.next();
                }
                DfsStateColor::Black => {
                    dfs = visitor.forward_or_cross_tr(s, tr);
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
