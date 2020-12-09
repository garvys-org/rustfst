use crate::fst_properties::FstProperties;
use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;

/// Plus-Sum weights of trs leaving the same state, going to the same state
/// and with the same input and output labels.
pub fn tr_sum<W: Semiring, F: MutableFst<W>>(ifst: &mut F) {
    let props = ifst.properties();
    unsafe {
        for s in ifst.states_range() {
            ifst.sum_trs_unchecked(s);
        }
    }
    let mut outprops = props
        & FstProperties::arcsort_properties()
        & FstProperties::delete_arcs_properties()
        & FstProperties::weight_invariant_properties();
    if ifst.num_states() == 0 {
        outprops |= FstProperties::null_properties();
    }
    ifst.set_properties_with_mask(outprops, FstProperties::all_properties());
}

#[cfg(test)]
mod test {
    use crate::fst_impls::VectorFst;
    use crate::fst_traits::MutableFst;
    use crate::semirings::{ProbabilityWeight, Semiring};
    use crate::Tr;
    use anyhow::Result;

    use super::*;

    #[test]
    fn test_tr_map_sum() -> Result<()> {
        let mut fst_in = VectorFst::<ProbabilityWeight>::new();

        let s1 = fst_in.add_state();
        let s2 = fst_in.add_state();

        fst_in.add_tr(s1, Tr::new(0, 0, ProbabilityWeight::new(0.3), s2))?;
        fst_in.add_tr(s1, Tr::new(0, 1, ProbabilityWeight::new(0.3), s2))?;
        fst_in.add_tr(s1, Tr::new(1, 0, ProbabilityWeight::new(0.3), s2))?;
        fst_in.add_tr(s1, Tr::new(0, 0, ProbabilityWeight::new(0.3), s2))?;
        fst_in.add_tr(s1, Tr::new(0, 0, ProbabilityWeight::new(0.1), s2))?;

        fst_in.set_start(s1)?;
        fst_in.set_final(s2, ProbabilityWeight::one())?;

        let mut fst_out = VectorFst::<ProbabilityWeight>::new();

        let s1 = fst_out.add_state();
        let s2 = fst_out.add_state();

        fst_out.add_tr(s1, Tr::new(0, 0, ProbabilityWeight::new(0.7), s2))?;
        fst_out.add_tr(s1, Tr::new(0, 1, ProbabilityWeight::new(0.3), s2))?;
        fst_out.add_tr(s1, Tr::new(1, 0, ProbabilityWeight::new(0.3), s2))?;

        fst_out.set_start(s1)?;
        fst_out.set_final(s2, ProbabilityWeight::one())?;

        tr_sum(&mut fst_in);

        assert_eq!(fst_in, fst_out);

        Ok(())
    }
}
