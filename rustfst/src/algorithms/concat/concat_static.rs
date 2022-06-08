use anyhow::Result;

use crate::fst_properties::mutable_properties::concat_properties;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{AllocableFst, ExpandedFst, MutableFst};
use crate::semirings::Semiring;
use crate::{StateId, Tr, Trs, EPS_LABEL};

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
/// # use rustfst::fst_traits::Fst;
/// # use rustfst::FstPath;
/// # use rustfst::algorithms::concat::concat;
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
    let props1 = fst_1.properties();
    let props2 = fst_2.properties();
    let start1 = fst_1.start();
    if start1.is_none() {
        return Ok(());
    }
    let numstates1 = fst_1.num_states();
    fst_1.reserve_states(fst_2.num_states());

    for s2 in fst_2.states_iter() {
        let s1 = fst_1.add_state();
        if let Some(final_weight) = unsafe { fst_2.final_weight_unchecked(s2) } {
            unsafe { fst_1.set_final_unchecked(s1, final_weight) };
        }
        unsafe { fst_1.reserve_trs_unchecked(s1, fst_2.num_trs_unchecked(s2)) };
        for tr in unsafe { fst_2.get_trs_unchecked(s2).trs() } {
            let mut new_tr = tr.clone();
            new_tr.nextstate += numstates1 as StateId;
            unsafe { fst_1.add_tr_unchecked(s1, new_tr) };
        }
    }

    let start2 = fst_2.start();
    for s1 in 0..(numstates1 as StateId) {
        if let Some(weight) = unsafe { fst_1.final_weight_unchecked(s1) } {
            if let Some(_start2) = start2 {
                unsafe {
                    fst_1.add_tr_unchecked(
                        s1,
                        Tr::new(
                            EPS_LABEL,
                            EPS_LABEL,
                            weight,
                            _start2 + (numstates1 as StateId),
                        ),
                    )
                };
            }
            unsafe { fst_1.delete_final_weight_unchecked(s1) };
        }
    }

    if start2.is_some() {
        fst_1.set_properties_with_mask(
            concat_properties(props1, props2, false),
            FstProperties::all_properties(),
        );
    }

    Ok(())
}
