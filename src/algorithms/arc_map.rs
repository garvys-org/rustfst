use crate::fst_traits::{FinalStatesIterator, Fst, MutableFst};
use crate::semirings::Semiring;
use crate::Arc;

/// The ArcMapper interfaces defines how arcs and final weights are mapped.
/// This is useful for implementing operations that do not change the number of
/// arcs.
pub trait ArcMapper<S: Semiring> {
    fn arc_map(&mut self, arc: &mut Arc<S>);
    fn final_weight_map(&mut self, weight: &mut S);
}

/// Maps an arc using a mapper function object. This function modifies its Fst input.
pub fn arc_map<F, M>(ifst: &mut F, mapper: &mut M)
where
    F: MutableFst,
    M: ArcMapper<F::W>,
{
    let states: Vec<_> = ifst.states_iter().collect();
    for state in states {
        for arc in ifst.arcs_iter_mut(state).unwrap() {
            mapper.arc_map(arc);
        }

        if let Some(w) = ifst.final_weight_mut(state) {
            mapper.final_weight_map(w)
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
