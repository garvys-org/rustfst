use crate::fst_traits::{ExpandedFst, MutableFst};

/// Plus-Sum weights of arcs leaving the same state, going to the same state
/// and with the same input and output labels.
pub fn arc_sum<F: MutableFst + ExpandedFst>(ifst: &mut F) {
    unsafe {
        for s in 0..ifst.num_states() {
            ifst.sum_arcs_unchecked(s);
        }
    }
}
