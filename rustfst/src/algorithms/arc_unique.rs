use std::cmp::Ordering;

use crate::fst_traits::ExpandedFst;
use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::Arc;

pub(crate) fn arc_compare<W: Semiring>(arc_1: &Arc<W>, arc_2: &Arc<W>) -> Ordering {
    if arc_1.ilabel < arc_2.ilabel {
        return Ordering::Less;
    }
    if arc_1.ilabel > arc_2.ilabel {
        return Ordering::Greater;
    }
    if arc_1.olabel < arc_2.olabel {
        return Ordering::Less;
    }
    if arc_1.olabel > arc_2.olabel {
        return Ordering::Greater;
    }
    if arc_1.nextstate < arc_2.nextstate {
        return Ordering::Less;
    }
    if arc_1.nextstate > arc_2.nextstate {
        return Ordering::Greater;
    }
    Ordering::Equal
}

pub fn arc_unique<F: MutableFst + ExpandedFst>(ifst: &mut F) {
    unsafe {
        for s in 0..ifst.num_states() {
            ifst.unique_arcs_unchecked(s);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::fst_impls::VectorFst;
    use crate::fst_traits::MutableFst;
    use crate::semirings::{ProbabilityWeight, Semiring};
    use crate::Arc;
    use failure::Fallible;

    use super::*;

    #[test]
    fn test_arc_map_unique() -> Fallible<()> {
        let mut fst_in = VectorFst::<ProbabilityWeight>::new();

        let s1 = fst_in.add_state();
        let s2 = fst_in.add_state();

        fst_in.add_arc(s1, Arc::new(0, 0, ProbabilityWeight::new(0.3), s2))?;
        fst_in.add_arc(s1, Arc::new(0, 1, ProbabilityWeight::new(0.3), s2))?;
        fst_in.add_arc(s1, Arc::new(1, 0, ProbabilityWeight::new(0.3), s2))?;
        fst_in.add_arc(s1, Arc::new(0, 0, ProbabilityWeight::new(0.3), s2))?;
        fst_in.add_arc(s1, Arc::new(0, 0, ProbabilityWeight::new(0.1), s2))?;

        fst_in.set_start(s1)?;
        fst_in.set_final(s2, ProbabilityWeight::one())?;

        let mut fst_out = VectorFst::<ProbabilityWeight>::new();

        let s1 = fst_out.add_state();
        let s2 = fst_out.add_state();

        fst_out.add_arc(s1, Arc::new(0, 0, ProbabilityWeight::new(0.3), s2))?;
        fst_out.add_arc(s1, Arc::new(0, 0, ProbabilityWeight::new(0.1), s2))?;
        fst_out.add_arc(s1, Arc::new(0, 1, ProbabilityWeight::new(0.3), s2))?;
        fst_out.add_arc(s1, Arc::new(1, 0, ProbabilityWeight::new(0.3), s2))?;

        fst_out.set_start(s1)?;
        fst_out.set_final(s2, ProbabilityWeight::one())?;

        arc_unique(&mut fst_in);

        assert_eq!(fst_in, fst_out);

        Ok(())
    }
}
