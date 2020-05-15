use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;

/// Plus-Sum weights of trs leaving the same state, going to the same state
/// and with the same input and output labels.
pub fn tr_sum<W: Semiring, F: MutableFst<W>>(ifst: &mut F) {
    unsafe {
        for s in 0..ifst.num_states() {
            ifst.sum_trs_unchecked(s);
        }
    }
}
