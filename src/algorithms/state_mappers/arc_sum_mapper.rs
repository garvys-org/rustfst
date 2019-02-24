use std::cmp::Ordering;

use itertools::Itertools;

use crate::algorithms::StateMapper;
use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::Arc;

pub struct ArcSumMapper {}

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

impl<F: MutableFst> StateMapper<F> for ArcSumMapper {
    fn map_final_weight(&self, _weight: Option<&mut F::W>) {}

    fn map_arcs(&self, fst: &mut F, state: usize) {
        let arcs = fst.pop_arcs(state).unwrap();
        let arcs: Vec<_> = arcs
            .into_iter()
            .sorted_by(arc_compare)
            .into_iter()
            .coalesce(|mut x, y| {
                if x.ilabel == y.ilabel && x.olabel == y.olabel && x.nextstate == y.nextstate {
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
