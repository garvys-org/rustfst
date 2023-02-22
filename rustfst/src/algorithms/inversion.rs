use crate::fst_properties::mutable_properties::invert_properties;
use crate::fst_properties::FstProperties;
use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;

/// Invert the transduction corresponding to an FST
/// by exchanging the FST's input and output labels.
///
/// # Example 1
/// ```
/// # use rustfst::fst;
/// # use rustfst::utils::{acceptor, transducer};
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::algorithms::invert;
/// let mut fst : VectorFst<IntegerWeight> = fst![2 => 3];
/// invert(&mut fst);
///
/// assert_eq!(fst, fst![3 => 2]);
/// ```
///
/// # Example 2
///
/// ## Input
///
/// ![invert_in](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/invert_in.svg?sanitize=true)
///
/// ## Invert
///
/// ![invert_out](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/invert_out.svg?sanitize=true)
///
pub fn invert<W: Semiring, F: MutableFst<W>>(fst: &mut F) {
    let props = fst.properties();

    for state in fst.states_range() {
        unsafe {
            let mut it_tr = fst.tr_iter_unchecked_mut(state);
            for idx_tr in 0..it_tr.len() {
                let tr = it_tr.get_unchecked(idx_tr);
                let ilabel = tr.ilabel;
                let olabel = tr.olabel;
                it_tr.set_labels_unchecked(idx_tr, olabel, ilabel);
            }
        }
    }

    fst.set_properties_with_mask(invert_properties(props), FstProperties::all_properties());
}
