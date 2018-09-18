// use fst::{MutableFst, ExpandedFst};
// use semirings::Semiring;

// pub fn project<W: Semiring, F: ExpandedFst<W> + MutableFst<W>>(fst: &mut F, project_input: bool) {
//     let N = fst.num_states();
//     for state_id in 0..N {
//         for arc in fst.arcs_iter_mut(&state_id) {
//             // if project_input {
//             //     arc.olabel = arc.ilabel;
//             // } else {
//             //     arc.ilabel = arc.olabel;
//             // }
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use semirings::integer_weight::IntegerWeight;
//     use fst::transducer;
//     use vector_fst::VectorFst;

//     #[test]
//     fn test_projection_input() {
//      let a = vec![1, 2, 3];
//      let b = vec![4, 5, 6];

//      let mut fst : VectorFst<IntegerWeight> = transducer(a.into_iter(), b.clone().into_iter());
//      project(&mut fst, true);

//      let ref_fst = transducer(b.clone().into_iter(), b.clone().into_iter());

//      assert_eq!(fst, ref_fst);

//     }
// }