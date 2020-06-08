use crate::fst_traits::{AllocableFst, ExpandedFst, Fst, MutableFst};
use crate::semirings::Semiring;
use crate::Trs;

/// Generic method to convert an Fst into any other types implementing the MutableFst trait.
pub fn fst_convert_from_ref<W, F1, F2>(ifst: &F1) -> F2
where
    W: Semiring,
    F1: Fst<W>,
    F2: MutableFst<W> + AllocableFst<W>,
{
    let mut ofst = F2::new();

    // TODO: If ExpandedFst is implemented, use fst.num_states()
    ofst.add_states(ifst.states_iter().count());

    if let Some(start) = ifst.start() {
        unsafe { ofst.set_start_unchecked(start) };

        for data in ifst.fst_iter() {
            unsafe {
                ofst.reserve_trs_unchecked(data.state_id, data.num_trs);
            }
            for tr in data.trs.trs() {
                unsafe { ofst.add_tr_unchecked(data.state_id, tr.clone()) };
            }

            if let Some(final_weight) = data.final_weight {
                unsafe { ofst.set_final_unchecked(data.state_id, final_weight.clone()) };
            }
        }
    }

    ofst.set_symts_from_fst(ifst);
    ofst.set_properties_with_mask(ifst.properties(), ifst.properties());

    ofst
}

/// Generic method to convert an Fst into any other types implementing the MutableFst trait.
pub fn fst_convert<W, F1, F2>(ifst: F1) -> F2
where
    W: Semiring,
    F1: ExpandedFst<W>,
    F2: MutableFst<W> + AllocableFst<W>,
{
    let mut ofst = F2::new();
    ofst.add_states(ifst.num_states());

    ofst.set_symts_from_fst(&ifst);
    let iprops = ifst.properties();

    if let Some(start) = ifst.start() {
        unsafe { ofst.set_start_unchecked(start) };

        for fst_iter_data in ifst.fst_into_iter() {
            unsafe {
                ofst.reserve_trs_unchecked(fst_iter_data.state_id, fst_iter_data.num_trs);
            }
            for tr in fst_iter_data.trs {
                unsafe { ofst.add_tr_unchecked(fst_iter_data.state_id, tr) }
            }

            if let Some(w) = fst_iter_data.final_weight {
                unsafe { ofst.set_final_unchecked(fst_iter_data.state_id, w) };
            }
        }
    }

    ofst.set_properties_with_mask(iprops, iprops);

    ofst
}
