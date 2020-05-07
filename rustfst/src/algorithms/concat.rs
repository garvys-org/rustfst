use anyhow::Result;

use crate::algorithms::ReplaceFst;
use crate::fst_traits::{
    AllocableFst, CoreFst, ExpandedFst, Fst, FstIterator, MutableFst, StateIterator,
};
use crate::semirings::Semiring;
use crate::tr::Tr;
use crate::{SymbolTable, EPS_LABEL};
use std::sync::Arc;

/// Performs the concatenation of two wFSTs. If `A` transduces string `x` to `y` with weight `a`
/// and `B` transduces string `w` to `v` with weight `b`, then their concatenation
/// transduces string `xw` to `yv` with weight `a ⊗ b`.
///
/// # Example 1
/// ```
/// # #[macro_use] extern crate rustfst;
/// # use rustfst::utils::transducer;
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::fst_traits::PathsIterator;
/// # use rustfst::FstPath;
/// # use rustfst::algorithms::concat;
/// # use anyhow::Result;
/// # use std::collections::HashSet;
/// # fn main() -> Result<()> {
/// let mut fst_a : VectorFst<IntegerWeight> = fst![2 => 3];
/// let fst_b : VectorFst<IntegerWeight> = fst![6 => 5];
///
/// concat(&mut fst_a, &fst_b)?;
/// let paths : HashSet<_> = fst_a.paths_iter().collect();
///
/// let mut paths_ref = HashSet::<FstPath<IntegerWeight>>::new();
/// paths_ref.insert(fst_path![2,6 => 3,5]);
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
/// ![concat_in_1](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/concat_in_1.svg?sanitize=true)
///
/// ## Input Fst 2
///
/// ![concat_in_2](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/concat_in_2.svg?sanitize=true)
///
/// ## Concat
///
/// ![concat_out](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/concat_out.svg?sanitize=true)
///
pub fn concat<W, F1, F2>(fst_1: &mut F1, fst_2: &F2) -> Result<()>
where
    W: Semiring,
    F1: ExpandedFst<W> + MutableFst<W> + AllocableFst<W>,
    F2: ExpandedFst<W>,
{
    let start1 = fst_1.start();
    if start1.is_none() {
        return Ok(());
    }
    let numstates1 = fst_1.num_states();
    fst_1.reserve_states(fst_2.num_states());

    for s2 in 0..fst_2.num_states() {
        let s1 = fst_1.add_state();
        if let Some(final_weight) = unsafe { fst_2.final_weight_unchecked(s2) } {
            unsafe { fst_1.set_final_unchecked(s1, final_weight.clone()) };
        }
        unsafe { fst_1.reserve_trs_unchecked(s1, fst_2.num_trs_unchecked(s2)) };
        for tr in unsafe { fst_2.tr_iter_unchecked(s2) } {
            let mut new_tr = tr.clone();
            new_tr.nextstate += numstates1;
            unsafe { fst_1.add_tr_unchecked(s1, new_tr) };
        }
    }

    let start2 = fst_2.start();
    for s1 in 0..numstates1 {
        if let Some(weight) = unsafe { fst_1.final_weight_unchecked(s1) } {
            if let Some(_start2) = start2 {
                let weight = weight.clone();
                unsafe {
                    fst_1.add_tr_unchecked(
                        s1,
                        Tr::new(EPS_LABEL, EPS_LABEL, weight, _start2 + numstates1),
                    )
                };
            }
            unsafe { fst_1.delete_final_weight_unchecked(s1) };
        }
    }

    Ok(())
}

/// Computes the concatenation (product) of two FSTs; this version is a delayed
/// FST. If FST1 transduces string x to y with weight a and FST2 transduces
/// string w to v with weight b, then their concatenation transduces string xw
/// to yv with Times(a, b).
#[derive(Debug, Clone, PartialEq)]
pub struct ConcatFst<W: Semiring, F: Fst<W> + 'static>(ReplaceFst<W, F, F>);
//
// impl<F: Fst + MutableFst + AllocableFst> ConcatFst<F>
// where
//     F::W: 'static,
// {
//     //TODO: Use a borrow and not a move
//     //TODO: Allow fsts of different types
//     pub fn new(fst1: F, fst2: F) -> Result<Self> {
//         let mut rfst = F::new();
//         rfst.add_states(3);
//         unsafe { rfst.set_start_unchecked(0) };
//         unsafe { rfst.set_final_unchecked(2, F::W::one()) };
//         if let Some(isymt) = fst1.input_symbols() {
//             rfst.set_input_symbols(Arc::clone(isymt));
//         }
//         if let Some(osymt) = fst1.output_symbols() {
//             rfst.set_output_symbols(Arc::clone(osymt));
//         }
//         unsafe { rfst.add_tr_unchecked(0, Tr::new(EPS_LABEL, std::usize::MAX, F::W::one(), 1)) };
//         unsafe {
//             rfst.add_tr_unchecked(1, Tr::new(EPS_LABEL, std::usize::MAX - 1, F::W::one(), 2))
//         };
//
//         let mut fst_tuples = Vec::with_capacity(3);
//         fst_tuples.push((0, rfst));
//         fst_tuples.push((std::usize::MAX, fst1));
//         fst_tuples.push((std::usize::MAX - 1, fst2));
//
//         Ok(ConcatFst(ReplaceFst::new(fst_tuples, 0, false)?))
//     }
// }
//
// impl<F: Fst> CoreFst for ConcatFst<F>
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
// impl<'a, F: Fst + 'static> StateIterator<'a> for ConcatFst<F>
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
// impl<'a, F: Fst + 'static> TrIterator<'a> for ConcatFst<F>
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
// impl<F: Fst + 'static> Fst for ConcatFst<F>
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
// impl<'a, F: Fst + 'static> FstIterator<'a> for ConcatFst<F>
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
