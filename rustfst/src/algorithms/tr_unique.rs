use std::cmp::Ordering;

use crate::fst_properties::FstProperties;
use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::Tr;

pub(crate) fn tr_compare<W: Semiring>(tr_1: &Tr<W>, tr_2: &Tr<W>) -> Ordering {
    if tr_1.ilabel < tr_2.ilabel {
        return Ordering::Less;
    }
    if tr_1.ilabel > tr_2.ilabel {
        return Ordering::Greater;
    }
    if tr_1.olabel < tr_2.olabel {
        return Ordering::Less;
    }
    if tr_1.olabel > tr_2.olabel {
        return Ordering::Greater;
    }
    //if tr_1.weight < tr_2.weight {
    //    return Ordering::Less;
    //}
    //if tr_1.weight > tr_2.weight {
    //    return Ordering::Greater;
    //}
    if tr_1.nextstate < tr_2.nextstate {
        return Ordering::Less;
    }
    if tr_1.nextstate > tr_2.nextstate {
        return Ordering::Greater;
    }
    Ordering::Equal
}

/// Keep a single instance of trs leaving the same state, going to the same state and
/// with the same input labels, output labels and weight.
pub fn tr_unique<W: Semiring, F: MutableFst<W>>(ifst: &mut F) {
    let props = ifst.properties();
    unsafe {
        for s in ifst.states_range() {
            ifst.unique_trs_unchecked(s);
        }
    }
    let mut outprops =
        props & FstProperties::arcsort_properties() & FstProperties::delete_arcs_properties();
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
    fn test_tr_map_unique() -> Result<()> {
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

        fst_out.add_tr(s1, Tr::new(0, 0, ProbabilityWeight::new(0.3), s2))?;
        fst_out.add_tr(s1, Tr::new(0, 0, ProbabilityWeight::new(0.1), s2))?;
        fst_out.add_tr(s1, Tr::new(0, 1, ProbabilityWeight::new(0.3), s2))?;
        fst_out.add_tr(s1, Tr::new(1, 0, ProbabilityWeight::new(0.3), s2))?;

        fst_out.set_start(s1)?;
        fst_out.set_final(s2, ProbabilityWeight::one())?;

        tr_unique(&mut fst_in);

        assert_eq!(fst_in, fst_out);

        Ok(())
    }

    //#[test]
    //fn test_tr_map_unique_1() -> Result<()> {
    //    let mut fst_in = VectorFst::<ProbabilityWeight>::new();

    //    let s1 = fst_in.add_state();
    //    let s2 = fst_in.add_state();

    //    fst_in.add_tr(s1, Tr::new(1, 2, ProbabilityWeight::new(1.0), s2))?;
    //    fst_in.add_tr(s1, Tr::new(1, 2, ProbabilityWeight::new(2.0), s2))?;
    //    fst_in.add_tr(s1, Tr::new(1, 2, ProbabilityWeight::new(1.0), s2))?;

    //    fst_in.set_start(s1)?;
    //    fst_in.set_final(s2, ProbabilityWeight::one())?;

    //    let mut fst_out = VectorFst::<ProbabilityWeight>::new();

    //    let s1 = fst_out.add_state();
    //    let s2 = fst_out.add_state();

    //    fst_out.add_tr(s1, Tr::new(1, 2, ProbabilityWeight::new(1.0), s2))?;
    //    fst_out.add_tr(s1, Tr::new(1, 2, ProbabilityWeight::new(2.0), s2))?;

    //    fst_out.set_start(s1)?;
    //    fst_out.set_final(s2, ProbabilityWeight::one())?;

    //    tr_unique(&mut fst_in);

    //    assert_eq!(fst_in, fst_out);

    //    Ok(())
    //}
}
