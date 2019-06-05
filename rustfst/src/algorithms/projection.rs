use crate::fst_traits::{ExpandedFst, MutableFst};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
/// Different types of labels projection in a FST.
pub enum ProjectType {
    /// Input projection : output labels are replaced with input ones.
    ProjectInput,
    /// Output projection : input labels are replaced with output ones.
    ProjectOutput,
}

/// This operation projects an FST onto its domain or range by either copying
/// each arc's input label to its output label or vice versa.
///
/// # Example : Project input
/// ```
/// # #[macro_use] extern crate rustfst;
/// # use failure::Fallible;
/// # use rustfst::utils::{acceptor, transducer};
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::algorithms::{project, ProjectType};
/// # fn main() -> Fallible<()> {
/// let mut fst : VectorFst<IntegerWeight> = fst![2 => 3];
/// project(&mut fst, ProjectType::ProjectInput);
///
/// assert_eq!(fst, fst![2]);
/// # Ok(())
/// # }
/// ```
///
/// # Example : Project output
/// ```
/// # #[macro_use] extern crate rustfst;
/// # use failure::Fallible;
/// # use rustfst::utils::{acceptor, transducer};
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::algorithms::{project, ProjectType};
/// # fn main() -> Fallible<()> {
/// let mut fst : VectorFst<IntegerWeight> = fst![2 => 3];
/// project(&mut fst, ProjectType::ProjectOutput);
///
/// assert_eq!(fst, fst![3]);
/// # Ok(())
/// # }
/// ```
pub fn project<F: ExpandedFst + MutableFst>(fst: &mut F, project_type: ProjectType) {
    match project_type {
        ProjectType::ProjectInput => {
            for state in 0..fst.num_states() {
                for arc in fst.arcs_iter_mut(state).unwrap() {
                    arc.olabel = arc.ilabel;
                }
            }
        }
        ProjectType::ProjectOutput => {
            for state in 0..fst.num_states() {
                for arc in fst.arcs_iter_mut(state).unwrap() {
                    arc.ilabel = arc.olabel;
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use counter::Counter;
    use failure::Fallible;

    use crate::fst_traits::PathsIterator;
    use crate::test_data::vector_fst::get_vector_fsts_for_tests;

    use super::*;

    #[test]
    fn test_projection_input_generic() -> Fallible<()> {
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

            project(&mut projected_fst, ProjectType::ProjectInput);
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
    fn test_projection_output_generic() -> Fallible<()> {
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

            project(&mut projected_fst, ProjectType::ProjectOutput);
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
