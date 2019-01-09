use crate::fst_traits::{FinalStatesIterator, Fst, MutableFst};
use crate::semirings::Semiring;
use crate::Arc;

pub trait ArcMapper<S1: Semiring, S2: Semiring> {
    fn arc_map(&mut self, arc: &Arc<S1>) -> Arc<S2>;
    fn weight_map(&mut self, weight: &S1) -> S2;
}

pub trait ArcMapperInplace<S: Semiring> {
    fn arc_map(&mut self, arc: &mut Arc<S>);
    fn weight_map(&mut self, weight: &mut S);
}

pub fn arc_map<F1, F2, M>(ifst: &F1, mapper: &mut M) -> F2
where
    F1: Fst,
    F2: MutableFst,
    M: ArcMapper<F1::W, F2::W>,
{
    let mut ofst = F2::new();

    // Add all the states from the ifst to the ofst.
    for _ in ifst.states_iter() {
        ofst.add_state();
    }

    if let Some(start_state) = ifst.start() {
        ofst.set_start(start_state).unwrap();
    }

    for state in ifst.states_iter() {
        for arc in ifst.arcs_iter(state).unwrap() {
            let new_arc = mapper.arc_map(arc);
            ofst.add_arc(state, new_arc).unwrap();
        }
    }

    for final_state in ifst.final_states_iter() {
        let new_final_weight = mapper.weight_map(&final_state.final_weight);
        ofst.set_final(final_state.state_id, new_final_weight)
            .unwrap();
    }

    ofst
}

pub fn arc_map_inplace<F, M>(ifst: &mut F, mapper: &mut M)
where
    F: MutableFst,
    M: ArcMapperInplace<F::W>,
{
    let states: Vec<_> = ifst.states_iter().collect();
    for state in states {
        for arc in ifst.arcs_iter_mut(state).unwrap() {
            mapper.arc_map(arc);
        }

        if let Some(w) = ifst.final_weight_mut(state) {
            mapper.weight_map(w)
        }
    }
}
