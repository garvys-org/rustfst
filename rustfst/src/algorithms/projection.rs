use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
/// Different types of labels projection in a FST.
pub enum ProjectType {
    /// Input projection : output labels are replaced with input ones.
    ProjectInput,
    /// Output projection : input labels are replaced with output ones.
    ProjectOutput,
}

/// This operation projects an FST onto its domain or range by either copying
/// each transition input label to its output label or vice versa.
/// # Example 1
///
/// ## Project input
/// ```
/// # #[macro_use] extern crate rustfst;
/// # use anyhow::Result;
/// # use rustfst::utils::{acceptor, transducer};
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::algorithms::{project, ProjectType};
/// # fn main() -> Result<()> {
/// let mut fst : VectorFst<IntegerWeight> = fst![2 => 3];
/// project(&mut fst, ProjectType::ProjectInput);
/// assert_eq!(fst, fst![2]);
/// # Ok(())
/// # }
/// ```
///
/// ## Project output
///
/// ```rust
/// # #[macro_use] extern crate rustfst;
/// # use anyhow::Result;
/// # use rustfst::utils::{acceptor, transducer};
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::algorithms::{project, ProjectType};
/// # fn main() -> Result<()> {
/// let mut fst : VectorFst<IntegerWeight> = fst![2 => 3];
/// project(&mut fst, ProjectType::ProjectOutput);
/// assert_eq!(fst, fst![3]);
/// # Ok(())
/// # }
/// ```
///
/// # Example 2
///
/// ## Input
///
/// ![project_in](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/project_in.svg?sanitize=true)
///
/// ## Project input
///
/// ![project_out_project-input](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/project_out_project-input.svg?sanitize=true)
///
/// ## Project output
///
/// ![project_out_project-input](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/project_out_project-output.svg?sanitize=true)
pub fn project<W: Semiring, F: MutableFst<W>>(fst: &mut F, project_type: ProjectType) {
    match project_type {
        ProjectType::ProjectInput => {
            for state in 0..fst.num_states() {
                for tr in unsafe { fst.tr_iter_unchecked_mut(state) } {
                    tr.olabel = tr.ilabel;
                }
            }
        }
        ProjectType::ProjectOutput => {
            for state in 0..fst.num_states() {
                for tr in unsafe { fst.tr_iter_unchecked_mut(state) } {
                    tr.ilabel = tr.olabel;
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::fst_properties::FstProperties;
    use crate::fst_traits::ExpandedFst;
    use crate::proptest_fst::proptest_fst;

    use super::*;

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
