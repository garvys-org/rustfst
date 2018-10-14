use arc::Arc;
use fst_traits::{CoreFst, ExpandedFst, FinalStatesIterator, MutableFst};
use semirings::Semiring;
use std::collections::HashMap;
use Result;
use StateId;

fn add_epsilon_arc_to_initial_state<F1, F2>(
    fst: &F1,
    mapping: &HashMap<StateId, StateId>,
    fst_out: &mut F2,
) -> Result<()>
where
    F1: ExpandedFst,
    F2: MutableFst,
{
    let start_state = fst_out.start().unwrap();
    if let Some(old_start_state_fst) = fst.start() {
        fst_out.add_arc(
            &start_state,
            Arc::new(
                0,
                0,
                <F2 as CoreFst>::W::one(),
                *mapping.get(&old_start_state_fst).unwrap(),
            ),
        )?;
    }
    Ok(())
}

fn set_new_final_states<W, F1, F2>(
    fst: &F1,
    mapping: &HashMap<StateId, StateId>,
    fst_out: &mut F2,
) -> Result<()>
where
    W: Semiring,
    F1: ExpandedFst<W = W>,
    F2: MutableFst<W = W>,
{
    for old_final_state in fst.final_states_iter() {
        let final_state = mapping.get(&old_final_state.state_id).ok_or_else(|| {
            format_err!(
                "Key {:?} doesn't exist in mapping",
                old_final_state.state_id
            )
        })?;
        fst_out.set_final(final_state, old_final_state.final_weight)?;
    }

    Ok(())
}

/// Performs the union of two wFSTs.  If A transduces string x to y with weight a
/// and B transduces string w to v with weight b, then their union transduces x to y
/// with weight a and w to v with weight b.
///
/// # Example
/// ```
/// use rustfst::utils::transducer;
/// use rustfst::semirings::{Semiring, IntegerWeight};
/// use rustfst::fst_impls::VectorFst;
/// use rustfst::fst_traits::PathsIterator;
/// use rustfst::path::Path;
/// use rustfst::algorithms::union;
/// use std::collections::HashSet;
///
/// let fst_a : VectorFst<IntegerWeight> = transducer(vec![2].into_iter(), vec![3].into_iter()).unwrap();
/// let fst_b : VectorFst<IntegerWeight> = transducer(vec![6].into_iter(), vec![5].into_iter()).unwrap();
///
/// let fst_res : VectorFst<IntegerWeight> = union(&fst_a, &fst_b).unwrap();
/// let paths : HashSet<_> = fst_res.paths_iter().collect();
///
/// let mut paths_ref = HashSet::new();
/// paths_ref.insert(Path::new(vec![2], vec![3], IntegerWeight::one()));
/// paths_ref.insert(Path::new(vec![6], vec![5], IntegerWeight::one()));
///
/// assert_eq!(paths, paths_ref);
/// ```
pub fn union<W, F1, F2, F3>(fst_1: &F1, fst_2: &F2) -> Result<F3>
where
    W: Semiring,
    F1: ExpandedFst<W = W>,
    F2: ExpandedFst<W = W>,
    F3: MutableFst<W = W>,
{
    let mut fst_out = F3::new();

    let start_state = fst_out.add_state();
    fst_out.set_start(&start_state)?;

    let mapping_states_fst_1 = fst_out.add_fst(fst_1)?;
    let mapping_states_fst_2 = fst_out.add_fst(fst_2)?;

    add_epsilon_arc_to_initial_state(fst_1, &mapping_states_fst_1, &mut fst_out)?;
    add_epsilon_arc_to_initial_state(fst_2, &mapping_states_fst_2, &mut fst_out)?;

    set_new_final_states(fst_1, &mapping_states_fst_1, &mut fst_out)?;
    set_new_final_states(fst_2, &mapping_states_fst_2, &mut fst_out)?;

    Ok(fst_out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use failure::ResultExt;
    use fst_impls::VectorFst;
    use fst_traits::PathsIterator;
    use itertools::Itertools;
    use semirings::IntegerWeight;
    use std::collections::HashSet;
    use test_data::vector_fst::get_vector_fsts_for_tests;

    #[test]
    fn test_union_generic() {
        for data in get_vector_fsts_for_tests().combinations(2) {
            let fst_1 = &data[0].fst;
            let fst_2 = &data[1].fst;

            let mut paths_ref: HashSet<_> = fst_1.paths_iter().collect();
            paths_ref.extend(fst_2.paths_iter());

            let union_fst: VectorFst<IntegerWeight> = union(fst_1, fst_2)
                .with_context(|_| {
                    format_err!(
                        "Error when performing union operation between {:?} and {:?}",
                        &data[0].name,
                        &data[1].name
                    )
                })
                .unwrap();
            let paths: HashSet<_> = union_fst.paths_iter().collect();

            assert_eq!(
                paths, paths_ref,
                "Test failing for union between {:?} and {:?}",
                &data[0].name, &data[1].name
            );
        }
    }
}
