use std::collections::HashMap;

use failure::Fallible;
use unsafe_unwrap::UnsafeUnwrap;

use crate::algorithms::all_pairs_shortest_distance;
use crate::algorithms::arc_sum;
use crate::arc::Arc;
use crate::fst_traits::{ExpandedFst, FinalStatesIterator, MutableFst};
use crate::semirings::{Semiring, StarSemiring};
use crate::EPS_LABEL;

// Compute the wFST derived from "fst" by keeping only the epsilon transitions
fn compute_fst_epsilon<W, F1, F2>(fst: &F1, keep_only_epsilon: bool) -> Fallible<F2>
where
    W: Semiring,
    F1: ExpandedFst<W = W>,
    F2: MutableFst<W = W> + ExpandedFst<W = W>,
{
    let mut fst_epsilon = F2::new();

    // Map old states id to new ones
    let mut mapping_states = HashMap::new();

    // First pass to add the necessary states
    for old_state_id in fst.states_iter() {
        let new_state_id = fst_epsilon.add_state();
        mapping_states.insert(old_state_id, new_state_id);
    }

    // Second pass to add the arcs
    for old_state_id in fst.states_iter() {
        for old_arc in fst.arcs_iter(old_state_id)? {
            let a = keep_only_epsilon && old_arc.ilabel == EPS_LABEL && old_arc.olabel == EPS_LABEL;
            let b =
                !(old_arc.ilabel == EPS_LABEL && old_arc.olabel == EPS_LABEL || keep_only_epsilon);

            if a || b {
                fst_epsilon.add_arc(
                    mapping_states[&old_state_id],
                    Arc::new(
                        old_arc.ilabel,
                        old_arc.olabel,
                        old_arc.weight.clone(),
                        mapping_states[&old_arc.nextstate],
                    ),
                )?;
            }
        }
    }

    if let Some(start_state) = fst.start() {
        fst_epsilon.set_start(mapping_states[&start_state])?;
    }

    for old_final_state in fst.final_states_iter() {
        fst_epsilon.set_final(
            mapping_states[&old_final_state.state_id],
            old_final_state.final_weight,
        )?;
    }
    Ok(fst_epsilon)
}

/// This operation removes epsilon-transitions (when both the input and
/// output labels are an epsilon) from a transducer. The result will be an
/// equivalent FST that has no such epsilon transitions.
///
/// # Example
/// ```
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::fst_traits::MutableFst;
/// # use rustfst::algorithms::rm_epsilon;
/// # use rustfst::Arc;
/// # use rustfst::EPS_LABEL;
/// let mut fst = VectorFst::new();
/// let s0 = fst.add_state();
/// let s1 = fst.add_state();
/// fst.add_arc(s0, Arc::new(32, 25, IntegerWeight::new(78), s1));
/// fst.add_arc(s1, Arc::new(EPS_LABEL, EPS_LABEL, IntegerWeight::new(13), s0));
/// fst.set_start(s0).unwrap();
/// fst.set_final(s0, IntegerWeight::new(5));
///
/// let fst_no_epsilon : VectorFst<_> = rm_epsilon(&fst).unwrap();
///
/// let mut fst_no_epsilon_ref = VectorFst::new();
/// let s0 = fst_no_epsilon_ref.add_state();
/// let s1 = fst_no_epsilon_ref.add_state();
/// fst_no_epsilon_ref.add_arc(s0, Arc::new(32, 25, IntegerWeight::new(78), s1));
/// fst_no_epsilon_ref.add_arc(s1, Arc::new(32, 25, IntegerWeight::new(78 * 13), s1));
/// fst_no_epsilon_ref.set_start(s0).unwrap();
/// fst_no_epsilon_ref.set_final(s0, IntegerWeight::new(5));
/// fst_no_epsilon_ref.set_final(s1, IntegerWeight::new(5 * 13));
///
/// assert_eq!(fst_no_epsilon, fst_no_epsilon_ref);
/// ```
pub fn rm_epsilon<W, F1, F2>(fst: &F1) -> Fallible<F2>
where
    W: StarSemiring,
    F1: ExpandedFst<W = W>,
    F2: MutableFst<W = W> + ExpandedFst<W = W>,
{
    let fst_epsilon: F2 = compute_fst_epsilon(fst, true)?;
    let dists_fst_epsilon = all_pairs_shortest_distance(&fst_epsilon)?;

    let mut eps_closures = vec![vec![]; fst_epsilon.num_states()];

    for p in fst_epsilon.states_iter() {
        for q in fst_epsilon.states_iter() {
            if p != q && dists_fst_epsilon[p][q] != W::zero() {
                eps_closures[p].push((q, &dists_fst_epsilon[p][q]));
            }
        }
    }

    let fst_no_epsilon: F2 = compute_fst_epsilon(fst, false)?;

    let mut output_fst = fst_no_epsilon.clone();

    for p in fst_no_epsilon.states_iter() {
        for (q, w_prime) in &eps_closures[p] {
            for arc in fst_no_epsilon.arcs_iter(*q)? {
                output_fst.add_arc(
                    p,
                    Arc::new(
                        arc.ilabel,
                        arc.olabel,
                        w_prime.times(&arc.weight)?,
                        arc.nextstate,
                    ),
                )?;
            }

            if unsafe { fst_no_epsilon.is_final_unchecked(*q) } {
                if !unsafe { fst_no_epsilon.is_final_unchecked(p) } {
                    output_fst.set_final(p, W::zero())?;
                }
                let rho_prime_p = unsafe { output_fst.final_weight_unchecked(p).unsafe_unwrap() };
                let rho_q = unsafe { fst_no_epsilon.final_weight_unchecked(*q).unsafe_unwrap() };
                let new_weight = rho_prime_p.plus(&w_prime.times(&rho_q)?)?;
                output_fst.set_final(p, new_weight)?;
            }
        }
    }

    arc_sum(&mut output_fst);

    Ok(output_fst)
}

#[cfg(test)]
mod tests {
    use counter::Counter;
    use failure::format_err;
    use failure::ResultExt;

    use crate::fst_impls::VectorFst;
    use crate::fst_traits::PathsIterator;
    use crate::semirings::IntegerWeight;
    use crate::test_data::vector_fst::get_vector_fsts_for_tests;

    use super::*;

    // TODO: Add test with epsilon arcs

    #[test]
    fn test_epsilon_removal_generic() -> Fallible<()> {
        for data in get_vector_fsts_for_tests() {
            let fst = &data.fst;

            let paths_ref: Counter<_> = fst.paths_iter().collect();

            let epsilon_removed_fst: VectorFst<IntegerWeight> =
                rm_epsilon(fst).with_context(|_| {
                    format_err!(
                        "Error when performing epsilon removal operation for wFST {:?}",
                        &data.name,
                    )
                })?;
            let paths: Counter<_> = epsilon_removed_fst.paths_iter().collect();

            assert_eq!(
                paths, paths_ref,
                "Test failing for epsilon removal for wFST {:?}",
                &data.name
            );
        }
        Ok(())
    }
}
