use fst_traits::{ExpandedFst, MutableFst};

pub fn project<F: ExpandedFst + MutableFst>(fst: &mut F, project_input: bool) {
    for state_id in 0..fst.num_states() {
        for arc in fst.arcs_iter_mut(&state_id) {
            if project_input {
                arc.olabel = arc.ilabel;
            } else {
                arc.ilabel = arc.olabel;
            }
        }
    }
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
        project(&mut fst, true);

        let ref_fst = transducer(a.clone().into_iter(), a.clone().into_iter()).unwrap();

        assert_eq!(fst, ref_fst);
    }

    #[test]
    fn test_projection_output() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];

        let mut fst: VectorFst<ProbabilityWeight> =
            transducer(a.clone().into_iter(), b.clone().into_iter()).unwrap();
        project(&mut fst, false);

        let ref_fst = transducer(b.clone().into_iter(), b.clone().into_iter()).unwrap();

        assert_eq!(fst, ref_fst);
    }
}
