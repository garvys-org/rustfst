use crate::fst_traits::{AllocableFst, ExpandedFst, Fst, MutableFst};

/// Generic method to convert an Fst into any other types implementing the MutableFst trait.
pub fn fst_convert_from_ref<F1, F2>(ifst: &F1) -> F2
where
    F1: Fst,
    F2: MutableFst<W = F1::W> + AllocableFst,
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
            for arc in data.arcs {
                unsafe { ofst.add_tr_unchecked(data.state_id, arc.clone()) };
            }

            if let Some(final_weight) = data.final_weight {
                unsafe { ofst.set_final_unchecked(data.state_id, final_weight.clone()) };
            }
        }
    }

    ofst.set_symts_from_fst(ifst);

    ofst
}

/// Generic method to convert an Fst into any other types implementing the MutableFst trait.
pub fn fst_convert<F1, F2>(ifst: F1) -> F2
where
    F1: ExpandedFst,
    F2: MutableFst<W = F1::W> + AllocableFst,
{
    let mut ofst = F2::new();
    ofst.add_states(ifst.num_states());

    ofst.set_symts_from_fst(&ifst);

    if let Some(start) = ifst.start() {
        unsafe { ofst.set_start_unchecked(start) };

        for fst_iter_data in ifst.fst_into_iter() {
            unsafe {
                ofst.reserve_trs_unchecked(fst_iter_data.state_id, fst_iter_data.num_trs);
            }
            for arc in fst_iter_data.arcs {
                unsafe { ofst.add_tr_unchecked(fst_iter_data.state_id, arc) }
            }

            if let Some(w) = fst_iter_data.final_weight {
                unsafe { ofst.set_final_unchecked(fst_iter_data.state_id, w) };
            }
        }
    }

    ofst
}
