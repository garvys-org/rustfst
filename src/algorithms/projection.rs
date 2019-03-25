use failure::Fallible;

use crate::algorithms::{ArcMapper, FinalArc, MapFinalAction};
use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::semirings::Semiring;
use crate::Arc;

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
    let mut mapper = ProjectMapper { project_type };
    fst.arc_map(&mut mapper).unwrap();
}

struct ProjectMapper {
    project_type: ProjectType,
}

impl<W: Semiring> ArcMapper<W> for ProjectMapper {
    fn arc_map(&mut self, arc: &mut Arc<W>) -> Fallible<()> {
        match self.project_type {
            ProjectType::ProjectInput => arc.olabel = arc.ilabel,
            ProjectType::ProjectOutput => arc.ilabel = arc.olabel,
        };
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
