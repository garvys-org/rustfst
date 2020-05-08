use std::cell::UnsafeCell;
use std::fmt;
use std::iter::{repeat, Map, Repeat, Zip};
use std::sync::Arc;
use std::slice::Iter as IterSlice;

use anyhow::Result;
use itertools::izip;

use crate::algorithms::cache::FstImpl;
use crate::fst_traits::{Fst, FstIterData, FstIterator, MutableFst, StateIterator};
use crate::prelude::CoreFst;
use crate::{StateId, SymbolTable, Tr};

pub struct LazyFst<IMPL> {
    fst_impl: UnsafeCell<IMPL>,
    isymt: Option<Arc<SymbolTable>>,
    osymt: Option<Arc<SymbolTable>>,
}

// impl<IMPL: FstImpl> LazyFst<IMPL> {
//     pub(crate) fn from_impl(
//         fst_impl: IMPL,
//         isymt: Option<Arc<SymbolTable>>,
//         osymt: Option<Arc<SymbolTable>>,
//     ) -> Self {
//         Self {
//             fst_impl: UnsafeCell::new(fst_impl),
//             isymt,
//             osymt,
//         }
//     }
//
//     fn num_known_states(&self) -> usize {
//         let ptr = self.fst_impl.get();
//         let fst_impl = unsafe { ptr.as_ref().unwrap() };
//         fst_impl.num_known_states()
//     }
//
//     pub fn compute<F: MutableFst<W = IMPL::W>>(&self) -> Result<F> {
//         let ptr = self.fst_impl.get();
//         let fst_impl = unsafe { ptr.as_mut().unwrap() };
//         let mut fst: F = fst_impl.compute()?;
//         if let Some(isymt) = &self.isymt {
//             fst.set_input_symbols(Arc::clone(isymt));
//         }
//         if let Some(osymt) = &self.osymt {
//             fst.set_output_symbols(Arc::clone(osymt));
//         }
//         Ok(fst)
//     }
// }
//
// impl<IMPL: FstImpl> CoreFst<IMPL::W> for LazyFst<IMPL> {
//     fn start(&self) -> Option<usize> {
//         let ptr = self.fst_impl.get();
//         let fst_impl = unsafe { ptr.as_mut().unwrap() };
//         fst_impl.start().unwrap()
//     }
//
//     fn final_weight(&self, state_id: usize) -> Result<Option<&Self::W>> {
//         let ptr = self.fst_impl.get();
//         let fst_impl = unsafe { ptr.as_mut().unwrap() };
//         fst_impl.final_weight(state_id)
//     }
//
//     unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<&Self::W> {
//         self.final_weight(state_id).unwrap()
//     }
//
//     fn num_trs(&self, s: usize) -> Result<usize> {
//         let ptr = self.fst_impl.get();
//         let fst_impl = unsafe { ptr.as_mut().unwrap() };
//         fst_impl.num_trs(s)
//     }
//
//     unsafe fn num_trs_unchecked(&self, s: usize) -> usize {
//         self.num_trs(s).unwrap()
//     }
// }
//
// impl<'a, IMPL: FstImpl> TrIterator<'a> for LazyFst<IMPL> {
//     type Iter = IterSlice<'a, Tr<IMPL::W>>;
//
//     fn tr_iter(&'a self, state_id: usize) -> Result<Self::Iter> {
//         let ptr = self.fst_impl.get();
//         let fst_impl = unsafe { ptr.as_mut().unwrap() };
//         fst_impl.tr_iter(state_id)
//     }
//
//     unsafe fn tr_iter_unchecked(&'a self, state_id: usize) -> Self::Iter {
//         self.tr_iter(state_id).unwrap()
//     }
// }
//
// #[derive(Clone)]
// pub struct StatesIteratorLazyFst<'a, T> {
//     pub(crate) fst: &'a T,
//     pub(crate) s: usize,
// }
//
// impl<'a, IMPL: FstImpl> Iterator for StatesIteratorLazyFst<'a, LazyFst<IMPL>> {
//     type Item = StateId;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.s < self.fst.num_known_states() {
//             let s_cur = self.s;
//             // Force expansion of the state
//             self.fst.tr_iter(s_cur).unwrap();
//             self.s += 1;
//             Some(s_cur)
//         } else {
//             None
//         }
//     }
// }
//
// impl<'a, IMPL: FstImpl + 'a> StateIterator<'a> for LazyFst<IMPL> {
//     type Iter = StatesIteratorLazyFst<'a, LazyFst<IMPL>>;
//
//     fn states_iter(&'a self) -> Self::Iter {
//         self.start();
//         StatesIteratorLazyFst { fst: &self, s: 0 }
//     }
// }
//
// impl<IMPL: FstImpl + 'static> Fst for LazyFst<IMPL> {
//     fn input_symbols(&self) -> Option<&Arc<SymbolTable>> {
//         self.isymt.as_ref()
//     }
//
//     fn output_symbols(&self) -> Option<&Arc<SymbolTable>> {
//         self.osymt.as_ref()
//     }
//
//     fn set_input_symbols(&mut self, symt: Arc<SymbolTable>) {
//         self.isymt = Some(symt)
//     }
//
//     fn set_output_symbols(&mut self, symt: Arc<SymbolTable>) {
//         self.osymt = Some(symt);
//     }
//
//     fn take_input_symbols(&mut self) -> Option<Arc<SymbolTable>> {
//         self.isymt.take()
//     }
//
//     fn take_output_symbols(&mut self) -> Option<Arc<SymbolTable>> {
//         self.osymt.take()
//     }
// }

impl<IMPL: FstImpl> std::fmt::Debug for LazyFst<IMPL> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_ref().unwrap() };
        write!(f, "LazyFst {{ {:?} }}", &fst_impl)
    }
}

// impl<'a, IMPL: FstImpl + 'a> FstIterator<'a> for LazyFst<IMPL> {
//     type TrsIter = <LazyFst<IMPL> as TrIterator<'a>>::Iter;
//     type FstIter = Map<
//         Zip<<LazyFst<IMPL> as StateIterator<'a>>::Iter, Repeat<&'a Self>>,
//         Box<dyn FnMut((StateId, &'a Self)) -> FstIterData<&'a IMPL::W, Self::TrsIter>>,
//     >;
//
//     fn fst_iter(&'a self) -> Self::FstIter {
//         let it = repeat(self);
//         izip!(self.states_iter(), it).map(Box::new(|(state_id, p): (StateId, &'a Self)| {
//             FstIterData {
//                 state_id,
//                 trs: p.tr_iter(state_id).unwrap(),
//                 final_weight: p.final_weight(state_id).unwrap(),
//                 num_trs: p.num_trs(state_id).unwrap(),
//             }
//         }))
//     }
// }

impl<IMPL: FstImpl + PartialEq> PartialEq for LazyFst<IMPL> {
    fn eq(&self, other: &Self) -> bool {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_ref().unwrap() };

        let ptr_other = other.fst_impl.get();
        let fst_impl_other = unsafe { ptr_other.as_ref().unwrap() };

        fst_impl.eq(fst_impl_other)
    }
}

impl<IMPL: FstImpl + Clone + 'static> Clone for LazyFst<IMPL> {
    fn clone(&self) -> Self {
        unimplemented!()
        // let ptr = self.fst_impl.get();
        // let fst_impl = unsafe { ptr.as_ref().unwrap() };
        // Self {
        //     fst_impl: UnsafeCell::new(fst_impl.clone()),
        //     isymt: self.input_symbols().cloned(),
        //     osymt: self.output_symbols().cloned(),
        // }
    }
}
