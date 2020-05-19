use std::marker::PhantomData;

use unsafe_unwrap::UnsafeUnwrap;

use crate::algorithms::dfs_visit::Visitor;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{ExpandedFst, Fst};
use crate::semirings::Semiring;
use crate::Tr;
use crate::{StateId, NO_STATE_ID};

pub struct SccVisitor<'a, W: Semiring, F: Fst<W>> {
    pub scc: Option<Vec<i32>>,
    pub access: Option<Vec<bool>>,
    pub coaccess: Vec<bool>,
    start: StateId,
    fst: &'a F,
    nstates: usize,
    dfnumber: Vec<i32>,
    lowlink: Vec<i32>,
    onstack: Vec<bool>,
    scc_stack: Vec<StateId>,
    pub nscc: i32,
    pub props: FstProperties,
    w: PhantomData<W>,
}

impl<'a, W: Semiring, F: 'a + ExpandedFst<W>> SccVisitor<'a, W, F> {
    pub fn new(fst: &'a F, compute_scc: bool, compute_acess: bool) -> Self {
        let n = fst.num_states();
        let mut props = FstProperties::empty();
        props |= FstProperties::ACYCLIC
            | FstProperties::INITIAL_ACYCLIC
            | FstProperties::ACCESSIBLE
            | FstProperties::COACCESSIBLE;
        props &= !(FstProperties::CYCLIC
            | FstProperties::INITIAL_CYCLIC
            | FstProperties::NOT_ACCESSIBLE
            | FstProperties::NOT_COACCESSIBLE);
        Self {
            scc: if compute_scc { Some(vec![-1; n]) } else { None },
            access: if compute_acess {
                Some(vec![false; n])
            } else {
                None
            },
            coaccess: vec![false; n],
            start: fst.start().unwrap_or(NO_STATE_ID),
            fst,
            nstates: 0,
            dfnumber: vec![-1; n],
            lowlink: vec![-1; n],
            onstack: vec![false; n],
            scc_stack: vec![],
            nscc: 0,
            props,
            w: PhantomData,
        }
    }
}

impl<'a, W: Semiring, F: 'a + ExpandedFst<W>> Visitor<'a, W, F> for SccVisitor<'a, W, F> {
    fn init_visit(&mut self, _fst: &'a F) {}

    fn init_state(&mut self, s: usize, root: usize) -> bool {
        self.scc_stack.push(s);
        self.dfnumber[s] = self.nstates as i32;
        self.lowlink[s] = self.nstates as i32;
        self.onstack[s] = true;
        if root == self.start {
            if let Some(ref mut access) = self.access {
                access[s] = true;
            }
        } else {
            if let Some(ref mut access) = self.access {
                access[s] = true;
            }
            self.props |= FstProperties::NOT_ACCESSIBLE;
            self.props &= !FstProperties::ACCESSIBLE;
        }
        self.nstates += 1;
        true
    }

    fn tree_tr(&mut self, _s: usize, _tr: &Tr<W>) -> bool {
        true
    }

    fn back_tr(&mut self, s: usize, tr: &Tr<W>) -> bool {
        let t = tr.nextstate;
        if self.dfnumber[t] < self.lowlink[s] {
            self.lowlink[s] = self.dfnumber[t];
        }
        if self.coaccess[t] {
            self.coaccess[s] = true;
        }
        self.props |= FstProperties::CYCLIC;
        self.props &= !FstProperties::ACYCLIC;
        if t == self.start {
            self.props |= FstProperties::INITIAL_CYCLIC;
            self.props &= !FstProperties::INITIAL_ACYCLIC;
        }
        true
    }

    fn forward_or_cross_tr(&mut self, s: usize, tr: &Tr<W>) -> bool {
        let t = tr.nextstate;
        if self.dfnumber[t] < self.dfnumber[s]
            && self.onstack[t]
            && self.dfnumber[t] < self.lowlink[s]
        {
            self.lowlink[s] = self.dfnumber[t];
        }
        if self.coaccess[t] {
            self.coaccess[s] = true;
        }
        true
    }

    #[inline]
    fn finish_state(&mut self, s: usize, parent: Option<usize>, _tr: Option<&Tr<W>>) {
        if unsafe { self.fst.is_final_unchecked(s) } {
            self.coaccess[s] = true;
        }
        if self.dfnumber[s] == self.lowlink[s] {
            let mut scc_coaccess = false;
            let mut i = self.scc_stack.len();
            let mut t;
            loop {
                i -= 1;
                t = self.scc_stack[i];
                if self.coaccess[t] {
                    scc_coaccess = true;
                }
                if s == t {
                    break;
                }
            }
            loop {
                t = unsafe { *self.scc_stack.last().unsafe_unwrap() };
                if let Some(ref mut scc) = self.scc {
                    scc[t] = self.nscc;
                }
                if scc_coaccess {
                    self.coaccess[t] = true;
                }
                self.onstack[t] = false;
                self.scc_stack.pop();
                if s == t {
                    break;
                }
            }
            if !scc_coaccess {
                self.props |= FstProperties::NOT_COACCESSIBLE;
                self.props &= !FstProperties::COACCESSIBLE;
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

    #[inline]
    fn finish_visit(&mut self) {
        if let Some(ref mut scc) = self.scc {
            for s in 0..scc.len() {
                scc[s] = self.nscc - 1 - scc[s];
            }
        }
    }
}
