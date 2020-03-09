use std::cell::UnsafeCell;
use std::fmt;
use std::iter::{repeat, Map, Repeat, Zip};
use std::rc::Rc;
use std::slice::Iter as IterSlice;

use failure::Fallible;
use itertools::izip;

use crate::algorithms::cache::FstImpl;
use crate::fst_traits::{ArcIterator, Fst, FstIterData, FstIterator, MutableFst, StateIterator};
use crate::prelude::CoreFst;
use crate::{Arc, StateId, SymbolTable};

pub struct DynamicFst<IMPL> {
    fst_impl: UnsafeCell<IMPL>,
    isymt: Option<Rc<SymbolTable>>,
    osymt: Option<Rc<SymbolTable>>,
}

impl<IMPL: FstImpl> DynamicFst<IMPL> {
    pub(crate) fn from_impl(
        fst_impl: IMPL,
        isymt: Option<Rc<SymbolTable>>,
        osymt: Option<Rc<SymbolTable>>,
    ) -> Self {
        Self {
            fst_impl: UnsafeCell::new(fst_impl),
            isymt,
            osymt,
        }
    }

    fn num_known_states(&self) -> usize {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_ref().unwrap() };
        fst_impl.num_known_states()
    }

    pub fn compute<F: MutableFst<W = IMPL::W>>(&mut self) -> Fallible<F> {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_mut().unwrap() };
        let mut fst: F = fst_impl.compute()?;
        if let Some(isymt) = &self.isymt {
            fst.set_input_symbols(Rc::clone(isymt));
        }
        if let Some(osymt) = &self.osymt {
            fst.set_output_symbols(Rc::clone(osymt));
        }
        Ok(fst)
    }
}

impl<IMPL: FstImpl> CoreFst for DynamicFst<IMPL> {
    type W = IMPL::W;

    fn start(&self) -> Option<usize> {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_mut().unwrap() };
        fst_impl.start().unwrap()
    }

    fn final_weight(&self, state_id: usize) -> Fallible<Option<&Self::W>> {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_mut().unwrap() };
        fst_impl.final_weight(state_id)
    }

    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<&Self::W> {
        self.final_weight(state_id).unwrap()
    }

    fn num_arcs(&self, s: usize) -> Fallible<usize> {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_mut().unwrap() };
        fst_impl.num_arcs(s)
    }

    unsafe fn num_arcs_unchecked(&self, s: usize) -> usize {
        self.num_arcs(s).unwrap()
    }
}

impl<'a, IMPL: FstImpl> ArcIterator<'a> for DynamicFst<IMPL> {
    type Iter = IterSlice<'a, Arc<IMPL::W>>;

    fn arcs_iter(&'a self, state_id: usize) -> Fallible<Self::Iter> {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_mut().unwrap() };
        fst_impl.arcs_iter(state_id)
    }

    unsafe fn arcs_iter_unchecked(&'a self, state_id: usize) -> Self::Iter {
        self.arcs_iter(state_id).unwrap()
    }
}

#[derive(Clone)]
pub struct StatesIteratorDynamicFst<'a, T> {
    pub(crate) fst: &'a T,
    pub(crate) s: usize,
}

impl<'a, IMPL: FstImpl> Iterator for StatesIteratorDynamicFst<'a, DynamicFst<IMPL>> {
    type Item = StateId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.s < self.fst.num_known_states() {
            let s_cur = self.s;
            // Force expansion of the state
            self.fst.arcs_iter(s_cur).unwrap();
            self.s += 1;
            Some(s_cur)
        } else {
            None
        }
    }
}

impl<'a, IMPL: FstImpl + 'a> StateIterator<'a> for DynamicFst<IMPL> {
    type Iter = StatesIteratorDynamicFst<'a, DynamicFst<IMPL>>;

    fn states_iter(&'a self) -> Self::Iter {
        self.start();
        StatesIteratorDynamicFst { fst: &self, s: 0 }
    }
}

impl<IMPL: FstImpl + 'static> Fst for DynamicFst<IMPL> {
    fn input_symbols(&self) -> Option<Rc<SymbolTable>> {
        self.isymt.clone()
    }

    fn output_symbols(&self) -> Option<Rc<SymbolTable>> {
        self.osymt.clone()
    }

    fn set_input_symbols(&mut self, symt: Rc<SymbolTable>) {
        self.isymt = Some(Rc::clone(&symt))
    }

    fn set_output_symbols(&mut self, symt: Rc<SymbolTable>) {
        self.osymt = Some(Rc::clone(&symt));
    }

    fn unset_input_symbols(&mut self) -> Option<Rc<SymbolTable>> {
        self.isymt.take()
    }

    fn unset_output_symbols(&mut self) -> Option<Rc<SymbolTable>> {
        self.osymt.take()
    }
}

impl<IMPL: FstImpl> std::fmt::Debug for DynamicFst<IMPL> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_ref().unwrap() };
        write!(f, "DynamicFst {{ {:?} }}", &fst_impl)
    }
}

impl<'a, IMPL: FstImpl + 'a> FstIterator<'a> for DynamicFst<IMPL> {
    type ArcsIter = <DynamicFst<IMPL> as ArcIterator<'a>>::Iter;
    type FstIter = Map<
        Zip<<DynamicFst<IMPL> as StateIterator<'a>>::Iter, Repeat<&'a Self>>,
        Box<dyn FnMut((StateId, &'a Self)) -> FstIterData<&'a IMPL::W, Self::ArcsIter>>,
    >;

    fn fst_iter(&'a self) -> Self::FstIter {
        let it = repeat(self);
        izip!(self.states_iter(), it).map(Box::new(|(state_id, p): (StateId, &'a Self)| {
            FstIterData {
                state_id,
                arcs: p.arcs_iter(state_id).unwrap(),
                final_weight: p.final_weight(state_id).unwrap(),
                num_arcs: p.num_arcs(state_id).unwrap(),
            }
        }))
    }
}

impl<IMPL: FstImpl + PartialEq> PartialEq for DynamicFst<IMPL> {
    fn eq(&self, other: &Self) -> bool {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_ref().unwrap() };

        let ptr_other = other.fst_impl.get();
        let fst_impl_other = unsafe { ptr_other.as_ref().unwrap() };

        fst_impl.eq(fst_impl_other)
    }
}

impl<IMPL: FstImpl + Clone + 'static> Clone for DynamicFst<IMPL> {
    fn clone(&self) -> Self {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_ref().unwrap() };
        Self {
            fst_impl: UnsafeCell::new(fst_impl.clone()),
            isymt: self.input_symbols(),
            osymt: self.output_symbols(),
        }
    }
}
