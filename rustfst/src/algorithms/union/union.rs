use anyhow::Result;
use unsafe_unwrap::UnsafeUnwrap;

use crate::fst_properties::mutable_properties::union_properties;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{AllocableFst, ExpandedFst, MutableFst};
use crate::semirings::Semiring;
use crate::tr::Tr;
use crate::{StateId, Trs, EPS_LABEL};

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
/// # use rustfst::fst_traits::Fst;
/// # use rustfst::FstPath;
/// # use rustfst::algorithms::union::union;
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
    let initial_acyclic_1 = fst_1
        .compute_and_update_properties(FstProperties::INITIAL_ACYCLIC)?
        .contains(FstProperties::INITIAL_ACYCLIC);
    let props1 = fst_1.properties();
    let props2 = fst_2.properties();
    let numstates1 = fst_1.num_states() as StateId;
    let start2 = fst_2.start();
    if start2.is_none() {
        return Ok(());
    }
    let start2 = unsafe { start2.unsafe_unwrap() };
    fst_1.reserve_states(fst_2.num_states() + if initial_acyclic_1 { 1 } else { 0 });

    for s2 in 0..(fst_2.num_states() as StateId) {
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
        fst_1.set_properties_with_mask(props2, FstProperties::copy_properties());
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
    fst_1.set_properties_with_mask(
        union_properties(props1, props2, false),
        FstProperties::all_properties(),
    );
    Ok(())
}
