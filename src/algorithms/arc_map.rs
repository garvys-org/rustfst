use crate::fst_traits::{FinalStatesIterator, Fst, MutableFst};
use crate::semirings::Semiring;
use crate::Arc;

pub trait ArcMapper<S: Semiring> {
    fn arc_map(&mut self, arc: &Arc<S>) -> Arc<S>;
    fn weight_map(&mut self, weight: &S) -> S;
}

pub trait ArcMapperMut<S: Semiring> {
    fn arc_map_mut(&mut self, arc: &mut Arc<S>);
    fn weight_map_mut(&mut self, weight: &mut S);
}

pub fn arc_map<W, F1, F2, M>(ifst: &F1, mapper: &mut M) -> F2
where
    W: Semiring,
    F1: Fst<W = W>,
    F2: MutableFst<W = W>,
    M: ArcMapper<W>,
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

pub fn arc_map_mut<F, M>(ifst: &mut F, mapper: &mut M)
where
    F: MutableFst,
    M: ArcMapperMut<F::W>,
{
    let states: Vec<_> = ifst.states_iter().collect();
    for state in states {
        for arc in ifst.arcs_iter_mut(state).unwrap() {
            mapper.arc_map_mut(arc);
        }

        if let Some(w) = ifst.final_weight_mut(state) {
            mapper.weight_map_mut(w)
        }
    }
}

pub fn convert_weights<F1, F2>(ifst: &F1) -> F2
where
    F1: Fst,
    F2: MutableFst,
    F1::W: Into<F2::W>,
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
            let new_arc = Arc::new(
                arc.ilabel,
                arc.olabel,
                arc.weight.clone().into(),
                arc.nextstate,
            );
            ofst.add_arc(state, new_arc).unwrap();
        }
    }

    for final_state in ifst.final_states_iter() {
        let new_final_weight = final_state.final_weight.into();
        ofst.set_final(final_state.state_id, new_final_weight)
            .unwrap();
    }

    ofst
}
