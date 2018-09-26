use fst_traits::{ExpandedFst, MutableFst};
use std::mem::swap;

pub fn invert<F: ExpandedFst + MutableFst>(fst: &mut F) {
    for state_id in 0..fst.num_states() {
        for arc in fst.arcs_iter_mut(&state_id) {
            swap(&mut arc.ilabel, &mut arc.olabel);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semirings::ProbabilityWeight;
    use utils::transducer;
    use fst_impls::VectorFst;

    #[test]
    fn test_invert() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];

        let mut fst: VectorFst<ProbabilityWeight> =
            transducer(a.clone().into_iter(), b.clone().into_iter());
        invert(&mut fst);

        let ref_fst = transducer(b.clone().into_iter(), a.clone().into_iter());

        assert_eq!(fst, ref_fst);
    }
}
