use arc::Arc;
use fst_traits::{ExpandedFst, FinalStatesIterator, MutableFst};
use semirings::Semiring;
use Result;
use EPS_LABEL;

/// Performs the concatenation of two wFSTs. If `A` transduces string `x` to `y` with weight `a`
/// and `B` transduces string `w` to `v` with weight `b`, then their concatenation
/// transduces string `xw` to `yv` with weight `a ⊗ b`.
///
/// # Example
/// ```
/// use rustfst::utils::transducer;
/// use rustfst::semirings::{Semiring, IntegerWeight};
/// use rustfst::fst_impls::VectorFst;
/// use rustfst::fst_traits::PathsIterator;
/// use rustfst::path::Path;
/// use rustfst::algorithms::concat;
/// use std::collections::HashSet;
///
/// let fst_a : VectorFst<IntegerWeight> = transducer(vec![2].into_iter(), vec![3].into_iter()).unwrap();
/// let fst_b : VectorFst<IntegerWeight> = transducer(vec![6].into_iter(), vec![5].into_iter()).unwrap();
///
/// let fst_res : VectorFst<IntegerWeight> = concat(&fst_a, &fst_b).unwrap();
/// let paths : HashSet<_> = fst_res.paths_iter().collect();
///
/// let mut paths_ref = HashSet::new();
/// paths_ref.insert(Path::new(vec![2, 6], vec![3, 5], IntegerWeight::one()));
///
/// assert_eq!(paths, paths_ref);
/// ```
pub fn concat<W, F1, F2, F3>(fst_1: &F1, fst_2: &F2) -> Result<F3>
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
        fst_out.set_start(&mapping_states_fst_1[&old_start_state])?;
    }

    // Final states of the first fst are connected to the start state of the second fst with an
    // epsilon transition
    if let Some(old_start_state_2) = fst_2.start() {
        let start_state_2 = &mapping_states_fst_2[&old_start_state_2];
        for old_final_state_1 in fst_1.final_states_iter() {
            let final_state_1 = &mapping_states_fst_1[&old_final_state_1.state_id];
            fst_out.add_arc(
                final_state_1,
                Arc::new(
                    EPS_LABEL,
                    EPS_LABEL,
                    old_final_state_1.final_weight,
                    *start_state_2,
                ),
            )?;
        }
    }

    // Final states are final states of the second fst
    for old_final_state in fst_2.final_states_iter() {
        let final_state = &mapping_states_fst_2[&old_final_state.state_id];
        fst_out.set_final(final_state, old_final_state.final_weight)?;
    }

    // FINISH

    Ok(fst_out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use counter::Counter;
    use failure::ResultExt;
    use fst_impls::VectorFst;
    use fst_traits::PathsIterator;
    use itertools::Itertools;
    use semirings::IntegerWeight;
    use test_data::vector_fst::get_vector_fsts_for_tests;
    use failure::format_err;

    #[test]
    fn test_concat_generic() {
        for data in get_vector_fsts_for_tests().combinations(2) {
            let fst_1 = &data[0].fst;
            let fst_2 = &data[1].fst;

            let mut paths_ref = Counter::new();
            for path_fst_1 in fst_1.paths_iter() {
                for path_fst_2 in fst_2.paths_iter() {
                    let mut new_path = path_fst_1.clone();
                    new_path.concat(path_fst_2);
                    paths_ref.update(vec![new_path]);
                }
            }

            let concat_fst: VectorFst<IntegerWeight> = concat(fst_1, fst_2)
                .with_context(|_| {
                    format_err!(
                        "Error when performing concat operation of {:?} and {:?}",
                        &data[0].name,
                        &data[1].name
                    )
                }).unwrap();

            let paths: Counter<_> = concat_fst.paths_iter().collect();

            assert_eq!(
                paths, paths_ref,
                "Test failing for concat between {:?} and {:?}",
                &data[0].name, &data[1].name
            );
        }
    }
}
