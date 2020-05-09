use std::sync::Arc;

use anyhow::Result;
use unsafe_unwrap::UnsafeUnwrap;

use crate::algorithms::ReplaceFst;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{
    AllocableFst, CoreFst, ExpandedFst, Fst, FstIterator, MutableFst, StateIterator,
};
use crate::semirings::Semiring;
use crate::tr::Tr;
use crate::{SymbolTable, EPS_LABEL, Trs};

/// Performs the union of two wFSTs. If A transduces string `x` to `y` with weight `a`
/// and `B` transduces string `w` to `v` with weight `b`, then their union transduces `x` to `y`
/// with weight `a` and `w` to `v` with weight `b`.
///
/// # Example 1
/// ```
/// # #[macro_use] extern crate rustfst;
/// # use anyhow::Result;
/// # use rustfst::utils::transducer;
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::fst_traits::PathsIterator;
/// # use rustfst::FstPath;
/// # use rustfst::algorithms::union;
/// # use std::collections::HashSet;
/// # fn main() -> Result<()> {
/// let mut fst_a : VectorFst<IntegerWeight> = fst![2 => 3];
/// let fst_b : VectorFst<IntegerWeight> = fst![6 => 5];
///
/// union(&mut fst_a, &fst_b)?;
/// let paths : HashSet<_> = fst_a.paths_iter().collect();
///
/// let mut paths_ref = HashSet::<FstPath<IntegerWeight>>::new();
/// paths_ref.insert(fst_path![2 => 3]);
/// paths_ref.insert(fst_path![6 => 5]);
///
/// assert_eq!(paths, paths_ref);
/// # Ok(())
/// # }
/// ```
///
/// # Example 2
///
/// ## Input Fst 1
///
/// ![union_in_1](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/union_in_1.svg?sanitize=true)
///
/// ## Input Fst 2
///
/// ![union_in_2](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/union_in_2.svg?sanitize=true)
///
/// ## Union
///
/// ![union_out](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/union_out.svg?sanitize=true)
///
pub fn union<W, F1, F2>(fst_1: &mut F1, fst_2: &F2) -> Result<()>
where
    W: Semiring,
    F1: AllocableFst<W> + MutableFst<W>,
    F2: ExpandedFst<W>,
{
    let numstates1 = fst_1.num_states();
    let fst_props_1 = fst_1.properties()?;
    let initial_acyclic_1 = fst_props_1.contains(FstProperties::INITIAL_ACYCLIC);
    let start2 = fst_2.start();
    if start2.is_none() {
        return Ok(());
    }
    let start2 = unsafe { start2.unsafe_unwrap() };
    fst_1.reserve_states(fst_2.num_states() + if initial_acyclic_1 { 1 } else { 0 });

    for s2 in 0..fst_2.num_states() {
        let s1 = fst_1.add_state();
        if let Some(final_weight) = unsafe { fst_2.final_weight_unchecked(s2) } {
            unsafe { fst_1.set_final_unchecked(s1, final_weight.clone()) };
        }
        unsafe { fst_1.reserve_trs_unchecked(s1, fst_2.num_trs_unchecked(s2)) };
        for tr in unsafe { fst_2.get_trs_unchecked(s2).trs() } {
            let mut new_tr = tr.clone();
            new_tr.nextstate += numstates1;
            unsafe { fst_1.add_tr_unchecked(s1, new_tr) };
        }
    }

    let start1 = fst_1.start();
    if start1.is_none() {
        unsafe { fst_1.set_start_unchecked(start2) };
        return Ok(());
    }
    let start1 = unsafe { start1.unsafe_unwrap() };

    if initial_acyclic_1 {
        unsafe {
            fst_1.add_tr_unchecked(
                start1,
                Tr::new(EPS_LABEL, EPS_LABEL, W::one(), start2 + numstates1),
            )
        };
    } else {
        let nstart1 = fst_1.add_state();
        unsafe { fst_1.set_start_unchecked(nstart1) };
        unsafe { fst_1.add_tr_unchecked(nstart1, Tr::new(EPS_LABEL, EPS_LABEL, W::one(), start1)) };
        unsafe {
            fst_1.add_tr_unchecked(
                nstart1,
                Tr::new(EPS_LABEL, EPS_LABEL, W::one(), start2 + numstates1),
            )
        };
    }
    Ok(())
}

/// Computes the union (sum) of two FSTs. This version is a delayed FST. If A
/// transduces string x to y with weight a and B transduces string w to v with
/// weight b, then their union transduces x to y with weight a and w to v with
/// weight b.
#[derive(Debug, Clone, PartialEq)]
pub struct UnionFst<W: Semiring, F: Fst<W> + 'static>(ReplaceFst<W, F, F>);
//
// impl<F: Fst + MutableFst + AllocableFst> UnionFst<F>
// where
//     F::W: 'static,
// {
//     //TODO: Use a borrow and not a move
//     //TODO: Allow fsts of different types
//     pub fn new(fst1: F, fst2: F) -> Result<Self> {
//         let mut rfst = F::new();
//         rfst.add_states(2);
//         rfst.set_start(0)?;
//         unsafe { rfst.set_final_unchecked(1, F::W::one()) };
//         if let Some(isymt) = fst1.input_symbols() {
//             rfst.set_input_symbols(Arc::clone(isymt));
//         }
//         if let Some(osymt) = fst1.output_symbols() {
//             rfst.set_output_symbols(Arc::clone(osymt));
//         }
//         unsafe { rfst.reserve_trs_unchecked(0, 2) };
//         unsafe { rfst.add_tr_unchecked(0, Tr::new(EPS_LABEL, std::usize::MAX, F::W::one(), 1)) };
//         unsafe {
//             rfst.add_tr_unchecked(0, Tr::new(EPS_LABEL, std::usize::MAX - 1, F::W::one(), 1))
//         };
//
//         let mut fst_tuples = Vec::with_capacity(3);
//         fst_tuples.push((0, rfst));
//         fst_tuples.push((std::usize::MAX, fst1));
//         fst_tuples.push((std::usize::MAX - 1, fst2));
//
//         Ok(UnionFst(ReplaceFst::new(fst_tuples, 0, false)?))
//     }
// }
//
// impl<F: Fst> CoreFst for UnionFst<F>
// where
//     F::W: 'static,
// {
//     type W = F::W;
//
//     fn start(&self) -> Option<usize> {
//         self.0.start()
//     }
//
//     fn final_weight(&self, state_id: usize) -> Result<Option<&Self::W>> {
//         self.0.final_weight(state_id)
//     }
//
//     unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<&Self::W> {
//         self.0.final_weight_unchecked(state_id)
//     }
//
//     fn num_trs(&self, s: usize) -> Result<usize> {
//         self.0.num_trs(s)
//     }
//
//     unsafe fn num_trs_unchecked(&self, s: usize) -> usize {
//         self.0.num_trs_unchecked(s)
//     }
// }
//
// impl<'a, F: Fst + 'static> StateIterator<'a> for UnionFst<F>
// where
//     F::W: 'static,
// {
//     type Iter = <ReplaceFst<F, F> as StateIterator<'a>>::Iter;
//
//     fn states_iter(&'a self) -> Self::Iter {
//         self.0.states_iter()
//     }
// }
//
// impl<'a, F: Fst + 'static> TrIterator<'a> for UnionFst<F>
// where
//     F::W: 'static,
// {
//     type Iter = <ReplaceFst<F, F> as TrIterator<'a>>::Iter;
//
//     fn tr_iter(&'a self, state_id: usize) -> Result<Self::Iter> {
//         self.0.tr_iter(state_id)
//     }
//
//     unsafe fn tr_iter_unchecked(&'a self, state_id: usize) -> Self::Iter {
//         self.0.tr_iter_unchecked(state_id)
//     }
// }
//
// impl<F: Fst + 'static> Fst for UnionFst<F>
// where
//     F::W: 'static,
// {
//     fn input_symbols(&self) -> Option<&Arc<SymbolTable>> {
//         self.0.input_symbols()
//     }
//
//     fn output_symbols(&self) -> Option<&Arc<SymbolTable>> {
//         self.0.output_symbols()
//     }
//
//     fn set_input_symbols(&mut self, symt: Arc<SymbolTable>) {
//         self.0.set_input_symbols(symt)
//     }
//
//     fn set_output_symbols(&mut self, symt: Arc<SymbolTable>) {
//         self.0.set_output_symbols(symt)
//     }
//
//     fn take_input_symbols(&mut self) -> Option<Arc<SymbolTable>> {
//         self.0.take_input_symbols()
//     }
//
//     fn take_output_symbols(&mut self) -> Option<Arc<SymbolTable>> {
//         self.0.take_output_symbols()
//     }
// }
//
// impl<'a, F: Fst + 'static> FstIterator<'a> for UnionFst<F>
// where
//     F::W: 'static,
// {
//     type TrsIter = <ReplaceFst<F, F> as FstIterator<'a>>::TrsIter;
//     type FstIter = <ReplaceFst<F, F> as FstIterator<'a>>::FstIter;
//
//     fn fst_iter(&'a self) -> Self::FstIter {
//         self.0.fst_iter()
//     }
// }
