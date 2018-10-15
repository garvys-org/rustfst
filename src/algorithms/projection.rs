use fst_traits::{ExpandedFst, MutableFst};
use Result;

/// This operation projects an FST onto its domain or range by either copying
/// each arc's input label to its output label or vice versa.
///
/// # Example : Project input
/// ```
/// use rustfst::utils::{acceptor, transducer};
/// use rustfst::semirings::{Semiring, IntegerWeight};
/// use rustfst::fst_impls::VectorFst;
/// use rustfst::algorithms::project;
///
/// let mut fst : VectorFst<IntegerWeight> = transducer(vec![2].into_iter(), vec![3].into_iter()).unwrap();
/// project(&mut fst, true).unwrap();
///
/// assert_eq!(fst, acceptor(vec![2].into_iter()).unwrap());
/// ```
///
/// # Example : Project output
/// ```
/// use rustfst::utils::{acceptor, transducer};
/// use rustfst::semirings::{Semiring, IntegerWeight};
/// use rustfst::fst_impls::VectorFst;
/// use rustfst::algorithms::project;
///
/// let mut fst : VectorFst<IntegerWeight> = transducer(vec![2].into_iter(), vec![3].into_iter()).unwrap();
/// project(&mut fst, false).unwrap();
///
/// assert_eq!(fst, acceptor(vec![3].into_iter()).unwrap());
/// ```
pub fn project<F: ExpandedFst + MutableFst>(fst: &mut F, project_input: bool) -> Result<()> {
    let states: Vec<_> = fst.states_iter().collect();
    for state_id in states {
        for arc in fst.arcs_iter_mut(&state_id)? {
            if project_input {
                arc.olabel = arc.ilabel;
            } else {
                arc.ilabel = arc.olabel;
            }
        }
    }

    Ok(())
}

/// This operation projects an FST onto its domain or range by copying
/// each arc's input label to its output label.
///
/// # Example
/// ```
/// use rustfst::utils::{acceptor, transducer};
/// use rustfst::semirings::{Semiring, IntegerWeight};
/// use rustfst::fst_impls::VectorFst;
/// use rustfst::algorithms::project;
///
/// let mut fst : VectorFst<IntegerWeight> = transducer(vec![2].into_iter(), vec![3].into_iter()).unwrap();
/// project(&mut fst, true).unwrap();
///
/// assert_eq!(fst, acceptor(vec![2].into_iter()).unwrap());
/// ```
pub fn project_input<F: ExpandedFst + MutableFst>(fst: &mut F) -> Result<()> {
    project(fst, true)
}

/// This operation projects an FST onto its domain or range by copying
/// each arc's output label to its input label.
///
/// # Example
/// ```
/// use rustfst::utils::{acceptor, transducer};
/// use rustfst::semirings::{Semiring, IntegerWeight};
/// use rustfst::fst_impls::VectorFst;
/// use rustfst::algorithms::project;
///
/// let mut fst : VectorFst<IntegerWeight> = transducer(vec![2].into_iter(), vec![3].into_iter()).unwrap();
/// project(&mut fst, false).unwrap();
///
/// assert_eq!(fst, acceptor(vec![3].into_iter()).unwrap());
/// ```
pub fn project_output<F: ExpandedFst + MutableFst>(fst: &mut F) -> Result<()> {
    project(fst, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use counter::Counter;
    use fst_traits::PathsIterator;
    use test_data::vector_fst::get_vector_fsts_for_tests;

    #[test]
    fn test_projection_input_generic() {
        for data in get_vector_fsts_for_tests() {
            let fst = &data.fst;

            let paths_ref: Counter<_> = fst
                .paths_iter()
                .map(|mut p| {
                    p.olabels = p.ilabels.clone();
                    p
                }).collect();

            let mut projected_fst = fst.clone();

            project_input(&mut projected_fst).unwrap();
            let paths: Counter<_> = projected_fst.paths_iter().collect();

            assert_eq!(
                paths, paths_ref,
                "Test failing for project_input on wFST {:?}",
                &data.name
            )
        }
    }

    #[test]
    fn test_projection_output_generic() {
        for data in get_vector_fsts_for_tests() {
            let fst = &data.fst;

            let paths_ref: Counter<_> = fst
                .paths_iter()
                .map(|mut p| {
                    p.ilabels = p.olabels.clone();
                    p
                }).collect();

            let mut projected_fst = fst.clone();

            project_output(&mut projected_fst).unwrap();
            let paths: Counter<_> = projected_fst.paths_iter().collect();

            assert_eq!(
                paths, paths_ref,
                "Test failing for project_output on wFST {:?}",
                &data.name
            )
        }
    }
}
