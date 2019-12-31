use crate::fst_traits::{ExpandedFst, MutableFst};

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
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
                for arc in unsafe { fst.arcs_iter_unchecked_mut(state) } {
                    arc.olabel = arc.ilabel;
                }
            }
        }
        ProjectType::ProjectOutput => {
            for state in 0..fst.num_states() {
                for arc in unsafe { fst.arcs_iter_unchecked_mut(state) } {
                    arc.ilabel = arc.olabel;
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::proptest_fst::proptest_fst;

    use crate::fst_properties::FstProperties;

    use super::*;

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_project_input_proptest(mut fst in proptest_fst()) {
            project(&mut fst, ProjectType::ProjectInput);
            prop_assume!(fst.properties().unwrap().intersects(FstProperties::ACCEPTOR));
        }
    }

    proptest! {
        #[test]
        fn test_project_output_proptest(mut fst in proptest_fst()) {
            project(&mut fst, ProjectType::ProjectOutput);
            prop_assume!(fst.properties().unwrap().intersects(FstProperties::ACCEPTOR));
        }
    }
}
