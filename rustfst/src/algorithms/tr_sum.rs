use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::fst_properties::FstProperties;

/// Plus-Sum weights of trs leaving the same state, going to the same state
/// and with the same input and output labels.
pub fn tr_sum<W: Semiring, F: MutableFst<W>>(ifst: &mut F) {
    let props = ifst.properties_revamp();
    unsafe {
        for s in 0..ifst.num_states() {
            ifst.sum_trs_unchecked(s);
        }
    }
    let mut outprops = props & FstProperties::arcsort_properties() & FstProperties::delete_arcs_properties() & FstProperties::weight_invariant_properties();
    if ifst.num_states() == 0 {
        outprops |= FstProperties::null_properties();
    }
    ifst.set_properties_with_mask(outprops, FstProperties::all_properties());
}
