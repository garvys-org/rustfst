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
    for state_id in 0..fst.num_states() {
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
    use fst_impls::VectorFst;
    use semirings::ProbabilityWeight;
    use utils::transducer;

    #[test]
    fn test_projection_input() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];

        let mut fst: VectorFst<ProbabilityWeight> =
            transducer(a.clone().into_iter(), b.clone().into_iter()).unwrap();
        project(&mut fst, true).unwrap();

        let ref_fst = transducer(a.clone().into_iter(), a.clone().into_iter()).unwrap();

        assert_eq!(fst, ref_fst);
    }

    #[test]
    fn test_projection_output() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];

        let mut fst: VectorFst<ProbabilityWeight> =
            transducer(a.clone().into_iter(), b.clone().into_iter()).unwrap();
        project(&mut fst, false).unwrap();

        let ref_fst = transducer(b.clone().into_iter(), b.clone().into_iter()).unwrap();

        assert_eq!(fst, ref_fst);
    }
}
