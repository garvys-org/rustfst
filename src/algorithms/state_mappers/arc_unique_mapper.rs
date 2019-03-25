use failure::Fallible;

use itertools::Itertools;

use crate::algorithms::state_mappers::arc_sum_mapper::arc_compare;
use crate::algorithms::StateMapper;
use crate::fst_traits::MutableFst;

/// Remove duplicate arcs leaving the same state, going to the same state
/// and with the same input and output labels.
pub struct ArcUniqueMapper {}

impl<F: MutableFst> StateMapper<F> for ArcUniqueMapper {
    fn map_final_weight(&self, _weight: Option<&mut F::W>) -> Fallible<()> {
        Ok(())
    }

    /// First sorts the exiting arcs by input label, output label and destination
    /// state and then uniques identical arcs.
    fn map_arcs(&self, fst: &mut F, state: usize) -> Fallible<()> {
        let arcs = fst.pop_arcs(state).unwrap();
        let arcs: Vec<_> = arcs
            .into_iter()
            .sorted_by(arc_compare)
            .into_iter()
            .coalesce(|x, y| {
                if x.ilabel == y.ilabel
                    && x.olabel == y.olabel
                    && x.nextstate == y.nextstate
                    && x.weight == y.weight
                {
                    Ok(x)
                } else {
                    Err((x, y))
                }
            })
            .collect();
        fst.reserve_arcs(state, arcs.len()).unwrap();
        arcs.into_iter()
            .for_each(|arc| fst.add_arc(state, arc).unwrap());
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::algorithms::state_map;
    use crate::fst_impls::VectorFst;
    use crate::fst_traits::MutableFst;
    use crate::semirings::{ProbabilityWeight, Semiring};
    use crate::Arc;
    use failure::Fallible;

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

        let mut mapper = ArcUniqueMapper {};
        state_map(&mut fst_in, &mut mapper)?;

        assert_eq!(fst_in, fst_out);

        Ok(())
    }
}
