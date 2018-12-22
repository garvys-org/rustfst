use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::Result;
use std::mem::swap;

/// This operation inverts the transduction corresponding to an FST by exchanging the FST's input and output labels.
///
/// # Example
/// ```
/// use rustfst::utils::{acceptor, transducer};
/// use rustfst::semirings::{Semiring, IntegerWeight};
/// use rustfst::fst_impls::VectorFst;
/// use rustfst::algorithms::invert;
///
/// let mut fst : VectorFst<IntegerWeight> = transducer(vec![2].into_iter(), vec![3].into_iter()).unwrap();
/// invert(&mut fst).unwrap();
///
/// assert_eq!(fst, transducer(vec![3].into_iter(), vec![2].into_iter()).unwrap());
/// ```
pub fn invert<F: ExpandedFst + MutableFst>(fst: &mut F) -> Result<()> {
    let states: Vec<_> = fst.states_iter().collect();
    for state_id in states {
        for arc in fst.arcs_iter_mut(state_id)? {
            swap(&mut arc.ilabel, &mut arc.olabel);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fst_traits::PathsIterator;
    use crate::test_data::vector_fst::get_vector_fsts_for_tests;
    use counter::Counter;

    #[test]
    fn test_invert_generic() -> Result<()> {
        for data in get_vector_fsts_for_tests() {
            let fst = &data.fst;

            let paths_ref: Counter<_> = fst
                .paths_iter()
                .map(|mut p| {
                    swap(&mut p.ilabels, &mut p.olabels);
                    p
                })
                .collect();

            let mut projected_fst = fst.clone();

            invert(&mut projected_fst)?;
            let paths: Counter<_> = projected_fst.paths_iter().collect();

            assert_eq!(
                paths, paths_ref,
                "Test failing for invert on wFST {:?}",
                &data.name
            )
        }
        Ok(())
    }
}
