use failure::Fallible;

use std::mem::swap;

use crate::algorithms::{ArcMapper, FinalArc, MapFinalAction};
use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::semirings::Semiring;
use crate::Arc;

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
/// let mut fst : VectorFst<IntegerWeight> = fst![2 => 3];
/// invert(&mut fst);
///
/// assert_eq!(fst, fst![3 => 2]);
/// ```
pub fn invert<F: ExpandedFst + MutableFst>(fst: &mut F) {
    let mut mapper = InvertMapper {};
    fst.arc_map(&mut mapper).unwrap();
}

struct InvertMapper {}

impl<W: Semiring> ArcMapper<W> for InvertMapper {
    fn arc_map(&mut self, arc: &mut Arc<W>) -> Fallible<()> {
        swap(&mut arc.ilabel, &mut arc.olabel);
        Ok(())
    }

    fn final_arc_map(&mut self, _final_arc: &mut FinalArc<W>) -> Fallible<()> {
        Ok(())
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use counter::Counter;

    use failure::Fallible;

    use crate::fst_traits::PathsIterator;
    use crate::test_data::vector_fst::get_vector_fsts_for_tests;

    #[test]
    fn test_invert_generic() -> Fallible<()> {
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
