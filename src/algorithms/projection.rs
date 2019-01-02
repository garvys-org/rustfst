use crate::fst_traits::{ExpandedFst, MutableFst};

/// This operation projects an FST onto its domain or range by either copying
/// each arc's input label to its output label or vice versa.
///
/// # Example : Project input
/// ```
/// # #[macro_use] extern crate rustfst;
/// # use rustfst::Result;
/// # use rustfst::utils::{acceptor, transducer};
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::project;
/// # fn main() -> Result<()> {
/// let mut fst : VectorFst<IntegerWeight> = transducer![2 => 3];
/// project(&mut fst, true);
///
/// assert_eq!(fst, acceptor![2]);
/// # Ok(())
/// # }
/// ```
///
/// # Example : Project output
/// ```
/// # #[macro_use] extern crate rustfst;
/// # use rustfst::Result;
/// # use rustfst::utils::{acceptor, transducer};
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::project;
/// # fn main() -> Result<()> {
/// let mut fst : VectorFst<IntegerWeight> = transducer![2 => 3];
/// project(&mut fst, false);
///
/// assert_eq!(fst, acceptor(vec![3].into_iter()));
/// # Ok(())
/// # }
/// ```
pub fn project<F: ExpandedFst + MutableFst>(fst: &mut F, project_input: bool) {
    let states: Vec<_> = fst.states_iter().collect();
    for state_id in states {
        // Can't fail
        for arc in fst.arcs_iter_mut(state_id).unwrap() {
            if project_input {
                arc.olabel = arc.ilabel;
            } else {
                arc.ilabel = arc.olabel;
            }
        }
    }
}

/// This operation projects an FST onto its domain or range by copying
/// each arc's input label to its output label.
///
/// # Example
/// ```
/// # #[macro_use] extern crate rustfst;
/// # use rustfst::Result;
/// # use rustfst::utils::{acceptor, transducer};
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::project_input;
/// # fn main() -> Result<()> {
/// let mut fst : VectorFst<IntegerWeight> = transducer![2 => 3];
/// project_input(&mut fst);
///
/// assert_eq!(fst, acceptor![2]);
/// # Ok(())
/// # }
/// ```
pub fn project_input<F: ExpandedFst + MutableFst>(fst: &mut F) {
    project(fst, true)
}

/// This operation projects an FST onto its domain or range by copying
/// each arc's output label to its input label.
///
/// # Example
/// ```
/// # #[macro_use] extern crate rustfst;
/// # use rustfst::Result;
/// # use rustfst::utils::{acceptor, transducer};
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::project_output;
/// # fn main() -> Result<()> {
/// let mut fst : VectorFst<IntegerWeight> = transducer![2 => 3];
/// project_output(&mut fst);
///
/// assert_eq!(fst, acceptor![3]);
/// # Ok(())
/// # }
/// ```
pub fn project_output<F: ExpandedFst + MutableFst>(fst: &mut F) {
    project(fst, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    use counter::Counter;

    use crate::fst_traits::PathsIterator;
    use crate::test_data::vector_fst::get_vector_fsts_for_tests;
    use crate::Result;

    #[test]
    fn test_projection_input_generic() -> Result<()> {
        for data in get_vector_fsts_for_tests() {
            let fst = &data.fst;

            let paths_ref: Counter<_> = fst
                .paths_iter()
                .map(|mut p| {
                    p.olabels = p.ilabels.clone();
                    p
                })
                .collect();

            let mut projected_fst = fst.clone();

            project_input(&mut projected_fst);
            let paths: Counter<_> = projected_fst.paths_iter().collect();

            assert_eq!(
                paths, paths_ref,
                "Test failing for project_input on wFST {:?}",
                &data.name
            )
        }
        Ok(())
    }

    #[test]
    fn test_projection_output_generic() -> Result<()> {
        for data in get_vector_fsts_for_tests() {
            let fst = &data.fst;

            let paths_ref: Counter<_> = fst
                .paths_iter()
                .map(|mut p| {
                    p.ilabels = p.olabels.clone();
                    p
                })
                .collect();

            let mut projected_fst = fst.clone();

            project_output(&mut projected_fst);
            let paths: Counter<_> = projected_fst.paths_iter().collect();

            assert_eq!(
                paths, paths_ref,
                "Test failing for project_output on wFST {:?}",
                &data.name
            )
        }
        Ok(())
    }
}
