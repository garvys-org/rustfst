use crate::fst_traits::{AllocableFst, ExpandedFst, MutableFst};

/// Generic method to convert an Fst into any other types implementing the MutableFst trait.
pub fn fst_convert<F1, F2>(ifst: &F1) -> F2
where
    F1: ExpandedFst,
    F2: MutableFst<W = F1::W> + ExpandedFst + AllocableFst,
{
    let mut ofst = F2::new();
    ofst.add_states(ifst.num_states());

    if let Some(start) = ifst.start() {
        unsafe { ofst.set_start_unchecked(start) };

        for s in 0..ifst.num_states() {
            // Preallocation
            unsafe {
                ofst.reserve_arcs_unchecked(s, ifst.num_arcs_unchecked(s));
            }
            for arc in unsafe { ifst.arcs_iter_unchecked(s) } {
                unsafe { ofst.add_arc_unchecked(s, arc.clone()) };
            }

            if let Some(final_weight) = unsafe { ifst.final_weight_unchecked(s) } {
                unsafe { ofst.set_final_unchecked(s, final_weight.clone()) };
            }
        }
    }

    ofst.set_symts_from_fst(ifst);

    ofst
}

/// Generic method to convert an Fst into any other types implementing the MutableFst trait.
pub fn fst_convert_2<F1, F2>(ifst: F1) -> F2
where
    F1: ExpandedFst,
    F2: MutableFst<W = F1::W> + ExpandedFst + AllocableFst,
{
    let mut ofst = F2::new();
    ofst.add_states(ifst.num_states());

    ofst.set_symts_from_fst(&ifst);

    if let Some(start) = ifst.start() {
        unsafe { ofst.set_start_unchecked(start) };

        for fst_iter_data in ifst.fst_into_iter() {
            unsafe {
                ofst.reserve_arcs_unchecked(fst_iter_data.state_id, fst_iter_data.num_arcs);
            }
            for arc in fst_iter_data.arcs {
                unsafe { ofst.add_arc_unchecked(fst_iter_data.state_id, arc) }
            }

            if let Some(w) = fst_iter_data.final_weight {
                unsafe { ofst.set_final_unchecked(fst_iter_data.state_id, w) };
            }
        }
    }

    ofst
}
