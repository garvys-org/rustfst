use failure::Fallible;

use crate::arc::Arc;
use crate::fst_traits::{ExpandedFst, FinalStatesIterator, MutableFst};
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
/// let fst_a : VectorFst<IntegerWeight> = fst![2 => 3];
/// let fst_b : VectorFst<IntegerWeight> = fst![6 => 5];
///
/// let fst_res : VectorFst<IntegerWeight> = concat(&fst_a, &fst_b)?;
/// let paths : HashSet<_> = fst_res.paths_iter().collect();
///
/// let mut paths_ref = HashSet::<FstPath<IntegerWeight>>::new();
/// paths_ref.insert(fst_path![2,6 => 3,5]);
///
/// assert_eq!(paths, paths_ref);
/// # Ok(())
/// # }
/// ```
pub fn concat<W, F1, F2, F3>(fst_1: &F1, fst_2: &F2) -> Fallible<F3>
where
    W: Semiring,
    F1: ExpandedFst<W = W>,
    F2: ExpandedFst<W = W>,
    F3: MutableFst<W = W>,
{
    let mut fst_out = F3::new();

    let mapping_states_fst_1 = fst_out.add_fst(fst_1)?;
    let mapping_states_fst_2 = fst_out.add_fst(fst_2)?;

    // Start state is the start state of the first fst
    if let Some(old_start_state) = fst_1.start() {
        fst_out.set_start(mapping_states_fst_1[&old_start_state])?;
    }

    // Final states of the first fst are connected to the start state of the second fst with an
    // epsilon transition
    if let Some(old_start_state_2) = fst_2.start() {
        let start_state_2 = &mapping_states_fst_2[&old_start_state_2];
        for old_final_state_1 in fst_1.final_states_iter() {
            let final_state_1 = mapping_states_fst_1[&old_final_state_1.state_id];
            fst_out.add_arc(
                final_state_1,
                Arc::new(
                    EPS_LABEL,
                    EPS_LABEL,
                    old_final_state_1.final_weight.clone(),
                    *start_state_2,
                ),
            )?;
        }
    }

    // Final states are final states of the second fst
    for old_final_state in fst_2.final_states_iter() {
        let final_state = mapping_states_fst_2[&old_final_state.state_id];
        fst_out.set_final(final_state, old_final_state.final_weight.clone())?;
    }

    // FINISH

    Ok(fst_out)
}

