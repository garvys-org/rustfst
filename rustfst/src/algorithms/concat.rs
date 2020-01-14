use failure::Fallible;

use crate::arc::Arc;
use crate::fst_traits::{ExpandedFst, MutableFst, AllocableFst};
use crate::semirings::Semiring;
use crate::EPS_LABEL;

/// Performs the concatenation of two wFSTs. If `A` transduces string `x` to `y` with weight `a`
/// and `B` transduces string `w` to `v` with weight `b`, then their concatenation
/// transduces string `xw` to `yv` with weight `a ⊗ b`.
///
/// # Example
/// ```
/// # #[macro_use] extern crate rustfst;
/// # use rustfst::utils::transducer;
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::fst_traits::PathsIterator;
/// # use rustfst::FstPath;
/// # use rustfst::algorithms::concat;
/// # use failure::Fallible;
/// # use std::collections::HashSet;
/// # fn main() -> Fallible<()> {
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
pub fn concat<W, F1, F2>(fst_1: &mut F1, fst_2: &F2) -> Fallible<()>
where
    W: Semiring,
    F1: ExpandedFst<W = W> + MutableFst<W=W> + AllocableFst<W=W>,
    F2: ExpandedFst<W = W>,
{
    let start1 = fst_1.start();
    if start1.is_none() {
        return Ok(())
    }
    let numstates1 = fst_1.num_states();
    fst_1.reserve_states(fst_2.num_states());

    for s2 in 0..fst_2.num_states() {
        let s1 = fst_1.add_state();
        if let Some(final_weight) = unsafe {fst_2.final_weight_unchecked(s2)} {
            unsafe {fst_1.set_final_unchecked(s1, final_weight.clone())};
        }
        unsafe {fst_1.reserve_arcs_unchecked(s1, fst_2.num_arcs_unchecked(s2))};
        for arc in unsafe {fst_2.arcs_iter_unchecked(s2)} {
            let mut new_arc = arc.clone();
            new_arc.nextstate += numstates1;
            unsafe {fst_1.add_arc_unchecked(s1, new_arc)};
        }
    }

    let start2 = fst_2.start();
    for s1 in 0..numstates1 {
        if let Some(weight) = unsafe {fst_1.final_weight_unchecked(s1)} {
            if let Some(_start2) = start2 {
                let weight = weight.clone();
                unsafe {fst_1.add_arc_unchecked(s1, Arc::new(EPS_LABEL, EPS_LABEL, weight, _start2 + numstates1))};
            }
            // TODO: Move to delete_final_weight_unchecked
            fst_1.delete_final_weight(s1)?;
        }
    }

    Ok(())
}
