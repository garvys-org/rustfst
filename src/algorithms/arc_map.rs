use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::Arc;

/// The ArcMapper interfaces defines how arcs and final weights are mapped.
/// This is useful for implementing operations that do not change the number of
/// arcs.
pub trait ArcMapper<S: Semiring> {
    /// How to modify the arcs.
    fn arc_map(&mut self, arc: &mut Arc<S>);

    /// How to mofify the final weights.
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
