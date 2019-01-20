use itertools::Itertools;

use crate::algorithms::isomorphic::arc_compare;
use crate::algorithms::StateMapper;
use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;

pub struct ArcSumMapper {}

impl<F: MutableFst> StateMapper<F> for ArcSumMapper {
    fn map_final_weight(&self, _weight: Option<&mut F::W>) {}

    fn map_arcs(&self, fst: &mut F, state: usize) {
        let arcs = fst.pop_arcs(state).unwrap();
        let arcs: Vec<_> = arcs
            .into_iter()
            .sorted_by(arc_compare)
            .into_iter()
            .coalesce(|mut x, y| {
                if x.ilabel == y.ilabel && x.olabel == y.olabel && x.weight == y.weight {
                    x.weight.plus_assign(y.weight);
                    Ok(x)
                } else {
                    Err((x, y))
                }
            })
            .collect();
        fst.reserve_arcs(state, arcs.len()).unwrap();
        arcs.into_iter()
            .for_each(|arc| fst.add_arc(state, arc).unwrap());
    }
}
