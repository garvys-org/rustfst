use std::cmp;

use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::tr::Tr;
use crate::Label;

/// Turns a list of input labels and output labels into a linear FST.
/// The only accepted path in the FST has for input `labels_input` and for output `labels_output`.
///
/// # Example
///
/// ```
/// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::semirings::{ProbabilityWeight, Semiring};
/// # use rustfst::utils::transducer;
/// # use rustfst::Tr;
/// let labels_input = vec![32, 43, 21];
/// let labels_output = vec![53, 18, 89];
///
/// let fst : VectorFst<ProbabilityWeight> = transducer(&labels_input, &labels_output, ProbabilityWeight::one());
///
/// assert_eq!(fst.num_states(), 4);
///
/// // The transducer function produces the same FST as the following code
///
/// let mut fst_ref = VectorFst::new();
/// let s1 = fst_ref.add_state();
/// let s2 = fst_ref.add_state();
/// let s3 = fst_ref.add_state();
/// let s4 = fst_ref.add_state();
///
/// fst_ref.set_start(s1).unwrap();
/// fst_ref.set_final(s4, ProbabilityWeight::one()).unwrap();
///
/// fst_ref.add_tr(s1, Tr::new(labels_input[0], labels_output[0], ProbabilityWeight::one(), s2)).unwrap();
/// fst_ref.add_tr(s2, Tr::new(labels_input[1], labels_output[1], ProbabilityWeight::one(), s3)).unwrap();
/// fst_ref.add_tr(s3, Tr::new(labels_input[2], labels_output[2], ProbabilityWeight::one(), s4)).unwrap();
///
/// assert_eq!(fst, fst_ref);
/// ```
pub fn transducer<W: Semiring, F: MutableFst<W>>(
    labels_input: &[Label],
    labels_output: &[Label],
    weight: W,
) -> F {
    let max_size = cmp::max(labels_input.len(), labels_output.len());

    let mut fst = F::new();
    let mut state_cour = fst.add_state();

    // Can't fail as the state has just been added
    fst.set_start(state_cour).unwrap();

    for idx in 0..max_size {
        let i = labels_input.get(idx).unwrap_or(&0);
        let o = labels_output.get(idx).unwrap_or(&0);

        let new_state = fst.add_state();

        // Can't fail as the state has just been added
        fst.add_tr(state_cour, Tr::new(*i, *o, W::one(), new_state))
            .unwrap();

        state_cour = new_state;
    }

    // Can't fail as the state has just been added
    fst.set_final(state_cour, weight).unwrap();

    fst
}

/// Turns a list of labels into a linear acceptor (FST with the same labels for both input and output).
/// The only accepted path in the acceptor will be `labels`.
///
/// # Example
///
/// ```
/// use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
/// use rustfst::fst_impls::VectorFst;
/// use rustfst::semirings::{ProbabilityWeight, Semiring};
/// use rustfst::utils::acceptor;
/// use rustfst::Tr;
///
/// let labels = vec![32, 43, 21];
///
/// let fst : VectorFst<ProbabilityWeight> = acceptor(&labels, ProbabilityWeight::one());
///
/// assert_eq!(fst.num_states(), 4);
///
/// // The acceptor function produces the same FST as the following code
///
/// let mut fst_ref = VectorFst::new();
/// let s1 = fst_ref.add_state();
/// let s2 = fst_ref.add_state();
/// let s3 = fst_ref.add_state();
/// let s4 = fst_ref.add_state();
///
/// fst_ref.set_start(s1).unwrap();
/// fst_ref.set_final(s4, ProbabilityWeight::one()).unwrap();
///
/// fst_ref.add_tr(s1, Tr::new(labels[0], labels[0], ProbabilityWeight::one(), s2)).unwrap();
/// fst_ref.add_tr(s2, Tr::new(labels[1], labels[1], ProbabilityWeight::one(), s3)).unwrap();
/// fst_ref.add_tr(s3, Tr::new(labels[2], labels[2], ProbabilityWeight::one(), s4)).unwrap();
///
/// assert_eq!(fst, fst_ref);
///
/// ```
pub fn acceptor<W: Semiring, F: MutableFst<W>>(labels: &[Label], weight: W) -> F {
    let mut fst = F::new();
    let mut state_cour = fst.add_state();

    // Can't fail as the state has just been added
    fst.set_start(state_cour).unwrap();

    for l in labels {
        let new_state = fst.add_state();

        // Can't fail as the state has just been added
        fst.add_tr(state_cour, Tr::new(*l, *l, W::one(), new_state))
            .unwrap();
        state_cour = new_state;
    }

    // Can't fail as the state has just been added
    fst.set_final(state_cour, weight).unwrap();

    fst
}

/// Creates a linear Fst containing the arguments.
///
/// There are multiple forms to this macro :
///
/// - Create an unweighted linear acceptor :
///
/// This will return a linear FST with one transition for each label given
/// (same input and output, weight one).
///
/// ```
/// # #[macro_use] extern crate rustfst; fn main() {
/// # use rustfst::utils;
/// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst, Fst};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::semirings::{ProbabilityWeight, Semiring};
/// # use rustfst::utils::acceptor;
/// # use rustfst::{Tr, FstPath};
/// let fst : VectorFst<ProbabilityWeight> = fst![1,2,3];
/// assert_eq!(fst.paths_iter().count(), 1);
/// assert_eq!(fst.paths_iter().next().unwrap(), fst_path![1,2,3]);
/// # }
/// ```
///
/// - Create an unweighted linear transducer from two list of labels :
///
/// The only accepted path in the FST has for input the first
/// list of labels and for output the second list of labels.
///
/// ```
/// # #[macro_use] extern crate rustfst; fn main() {
/// # use rustfst::utils;
/// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst, Fst};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::semirings::{ProbabilityWeight, Semiring};
/// # use rustfst::utils::transducer;
/// # use rustfst::{Tr, FstPath};
/// let fst : VectorFst<ProbabilityWeight> = fst![1,2,3 => 1,2,4];
/// assert_eq!(fst.paths_iter().count(), 1);
/// assert_eq!(fst.paths_iter().next().unwrap(), fst_path![1,2,3 => 1,2,4]);
/// # }
/// ```
///
/// - Create a weighted linear acceptor :
///
/// This will return a linear FST with one transition for each label given
/// (same input and output, weight one).
///
/// ```
/// # #[macro_use] extern crate rustfst; fn main() {
/// # use rustfst::utils;
/// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst, Fst};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::semirings::{ProbabilityWeight, Semiring};
/// # use rustfst::utils::acceptor;
/// # use rustfst::{Tr, FstPath};
/// let fst : VectorFst<ProbabilityWeight> = fst![1,2,3; 0.2];
/// assert_eq!(fst.paths_iter().count(), 1);
/// assert_eq!(fst.paths_iter().next().unwrap(), fst_path![1,2,3; 0.2]);
/// # }
/// ```
///
/// - Create a weighted linear transducer from two list of labels and a weight :
///
/// The only accepted path in the FST has for input the first
/// list of labels and for output the second list of labels.
///
/// ```
/// # #[macro_use] extern crate rustfst; fn main() {
/// # use rustfst::utils;
/// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst, Fst};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::semirings::{ProbabilityWeight, Semiring};
/// # use rustfst::utils::transducer;
/// # use rustfst::{Tr, FstPath};
/// let fst : VectorFst<ProbabilityWeight> = fst![1,2,3 => 1,2,4; 0.2];
/// assert_eq!(fst.paths_iter().count(), 1);
/// assert_eq!(fst.paths_iter().next().unwrap(), fst_path![1,2,3 => 1,2,4; 0.2]);
/// # }
/// ```
///
#[macro_export]
macro_rules! fst {
    ( $( $x:expr ),* ) => {
        {
            fn semiring_one<W: Semiring>() -> W {
                W::one()
            }
            acceptor(
                &[$($x),*],
                semiring_one()
            )
        }
    };
    ( $( $x:expr ),* => $( $y:expr ),* ) => {
        {
            fn semiring_one<W: Semiring>() -> W {
                W::one()
            }
            transducer(
                &[$($x),*],
                &[$($y),*],
                semiring_one()
            )
        }
    };
    ( $( $x:expr ),* ; $weight:expr ) => {
        {
            fn semiring_new<W: Semiring>(v: W::Type) -> W {
                W::new(v)
            }
            acceptor(
                &[$($x),*],
                semiring_new($weight)
            )
        }
    };
    ( $( $x:expr ),* => $( $y:expr ),* ; $weight:expr ) => {
        {
            fn semiring_new<W: Semiring>(v: W::Type) -> W {
                W::new(v)
            }
            transducer(
                &[$($x),*],
                &[$($y),*],
                semiring_new($weight)
            )
        }
    };
}
