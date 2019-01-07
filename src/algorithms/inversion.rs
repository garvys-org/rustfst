use std::mem::swap;

use crate::fst_traits::{ExpandedFst, MutableFst};

/// This operation inverts the transduction corresponding to an FST
/// by exchanging the FST's input and output labels.
///
/// # Example
/// ```
/// # #[macro_use] extern crate rustfst;
/// # use rustfst::utils::{acceptor, transducer};
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::algorithms::invert;
/// let mut fst : VectorFst<IntegerWeight> = transducer![2 => 3];
/// invert(&mut fst);
///
/// assert_eq!(fst, transducer![3 => 2]);
/// ```
pub fn invert<F: ExpandedFst + MutableFst>(fst: &mut F) {
    let states: Vec<_> = fst.states_iter().collect();
    for state_id in states {
        // Can't fail
        for arc in fst.arcs_iter_mut(state_id).unwrap() {
            swap(&mut arc.ilabel, &mut arc.olabel);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fst_traits::PathsIterator;
    use crate::test_data::vector_fst::get_vector_fsts_for_tests;
    use crate::Result;
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

            invert(&mut projected_fst);
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
