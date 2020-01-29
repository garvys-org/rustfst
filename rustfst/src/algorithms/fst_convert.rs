use crate::fst_traits::{AllocableFst, ExpandedFst, MutableFst};

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
