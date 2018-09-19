use fst::{ExpandedFst, MutableFst};
use semirings::Semiring;

pub fn inverse<W: Semiring, F: ExpandedFst<W> + MutableFst<W>>(fst: &mut F) {
    for state_id in 0..fst.num_states() {
        for arc in fst.arcs_iter_mut(&state_id) {
            let old_olabel = arc.olabel;
            arc.olabel = arc.ilabel;
            arc.ilabel = old_olabel;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fst::transducer;
    use semirings::integer_weight::IntegerWeight;
    use vector_fst::VectorFst;

    #[test]
    fn test_projection_input() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];

        let mut fst: VectorFst<IntegerWeight> =
            transducer(a.clone().into_iter(), b.clone().into_iter());
        inverse(&mut fst);

        let ref_fst = transducer(b.clone().into_iter(), a.clone().into_iter());

        assert_eq!(fst, ref_fst);
    }
}
