use crate::fst_traits::{ExpandedFst, Fst};
use crate::semirings::Semiring;
use crate::tr::Tr;
use crate::{StateId, Trs};

use crate::algorithms::tr_filters::TrFilter;
use std::marker::PhantomData;
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

pub trait Visitor<'a, W: Semiring, F: Fst<W>> {
    /// Invoked before DFS visit.
    fn init_visit(&mut self, fst: &'a F);

    /// Invoked when state discovered (2nd arg is DFS tree root).
    fn init_state(&mut self, s: StateId, root: StateId) -> bool;

    /// Invoked when tree transition to white/undiscovered state examined.
    fn tree_tr(&mut self, s: StateId, tr: &Tr<W>) -> bool;

    /// Invoked when back transition to grey/unfinished state examined.
    fn back_tr(&mut self, s: StateId, tr: &Tr<W>) -> bool;

    /// Invoked when forward or cross transition to black/finished state examined.
    fn forward_or_cross_tr(&mut self, s: StateId, tr: &Tr<W>) -> bool;

    /// Invoked when state finished ('s' is tree root, 'parent' is kNoStateId,
    /// and 'tr' is nullptr).
    fn finish_state(&mut self, s: StateId, parent: Option<StateId>, tr: Option<&Tr<W>>);

    /// Invoked after DFS visit.
    fn finish_visit(&mut self);
}

struct DfsState<W, TRS>
where
    W: Semiring,
    TRS: Trs<W>,
{
    state_id: StateId,
    tr_iter: OpenFstIterator<W, TRS>,
    w: PhantomData<W>,
}

impl<W: Semiring, TRS: Trs<W>> DfsState<W, TRS> {
    #[inline]
    pub fn new<F: Fst<W, TRS = TRS>>(fst: &F, s: StateId) -> Self {
        Self {
            state_id: s,
            tr_iter: OpenFstIterator::new(unsafe { fst.get_trs_unchecked(s) }),
            w: PhantomData,
        }
    }
}

struct OpenFstIterator<W: Semiring, TRS: Trs<W>> {
    trs: TRS,
    pos: usize,
    w: PhantomData<W>,
}

impl<W: Semiring, TRS: Trs<W>> OpenFstIterator<W, TRS> {
    #[inline]
    fn new(trs: TRS) -> Self {
        Self {
            trs,
            pos: 0,
            w: PhantomData,
        }
    }

    #[inline]
    fn value(&self) -> &Tr<W> {
        unsafe { self.trs.trs().get_unchecked(self.pos) }
    }

    #[inline]
    fn done(&self) -> bool {
        let n = self.trs.trs().len();
        self.pos >= n
    }

    #[inline]
    fn next(&mut self) {
        self.pos += 1;
    }
}

pub fn dfs_visit<'a, W: Semiring, F: ExpandedFst<W>, V: Visitor<'a, W, F>, A: TrFilter<W>>(
    fst: &'a F,
    visitor: &mut V,
    tr_filter: &A,
    access_only: bool,
) {
    visitor.init_visit(fst);
    let start = match fst.start() {
        None => {
            visitor.finish_visit();
            return;
        }
        Some(s) => s,
    };

    let nstates = fst.num_states();
    let mut state_color = vec![DfsStateColor::White; nstates];
    let mut state_stack = vec![];

    // Continue dfs while true.
    let mut dfs = true;
    let mut root = start;
    loop {
        if !dfs || (root as usize) >= nstates {
            break;
        }
        state_color[root as usize] = DfsStateColor::Grey;
        state_stack.push(DfsState::new(fst, root));
        dfs = visitor.init_state(root, root);
        let mut state_stack_next = None;
        while !state_stack.is_empty() {
            let dfs_state = unsafe { state_stack.last_mut().unsafe_unwrap() };
            let s = dfs_state.state_id;
            let aiter = &mut dfs_state.tr_iter;
            if !dfs || aiter.done() {
                state_color[s as usize] = DfsStateColor::Black;
                state_stack.pop();
                if !state_stack.is_empty() {
                    let parent_state = unsafe { state_stack.last_mut().unsafe_unwrap() };
                    let piter = &mut parent_state.tr_iter;
                    visitor.finish_state(s, Some(parent_state.state_id), Some(piter.value()));
                    piter.next();
                } else {
                    visitor.finish_state(s, None, None);
                }
                continue;
            }
            let tr = aiter.value();
            let next_color = state_color[tr.nextstate as usize];
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
                    state_color[tr.nextstate as usize] = DfsStateColor::Grey;
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

        while (root as usize) < nstates && state_color[root as usize] != DfsStateColor::White {
            root += 1;
        }
    }
    visitor.finish_visit();
}
