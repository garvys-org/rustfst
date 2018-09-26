use arc::Arc;
use fst_traits::{CoreFst, ExpandedFst};
use std::collections::HashMap;
use Label;
use StateId;

/// Trait defining the methods to modify a wFST
pub trait MutableFst: CoreFst + for<'a> MutableArcIterator<'a> {

    /// Creates an empty wFST
    fn new() -> Self;

    /// The state with identifier passed as parameter is now the start state.
    /// This method should be called only once as there is only one start state 
    /// allowed in this implementation.
    ///
    fn set_start(&mut self, &StateId);
    fn add_state(&mut self) -> StateId;
    fn del_state(&mut self, &StateId);
    fn del_states<T: IntoIterator<Item = StateId>>(&mut self, states: T);
    fn add_arc(
        &mut self,
        source: &StateId,
        target: &StateId,
        ilabel: Label,
        olabel: Label,
        weight: <Self as CoreFst>::W,
    );
    fn set_final(&mut self, id: &StateId, finalweight: <Self as CoreFst>::W);
    // fn set_isyms<T: IntoIterator<Item=String>>(&mut self, symtab: T);
    // fn set_osyms<T: IntoIterator<Item=String>>(&mut self, symtab: T);

    fn add_fst<F: ExpandedFst<W = Self::W>>(
        &mut self,
        fst_to_add: &F,
    ) -> HashMap<StateId, StateId> {
        // Map old states id to new ones
        let mut mapping_states = HashMap::new();

        // First pass to add the necessary states
        for old_state_id in fst_to_add.states_iter() {
            let new_state_id = self.add_state();
            mapping_states.insert(old_state_id, new_state_id);
        }

        // Second pass to add the arcs
        for old_state_id in fst_to_add.states_iter() {
            for old_arc in fst_to_add.arcs_iter(&old_state_id) {
                self.add_arc(
                    &mapping_states[&old_state_id],
                    &mapping_states[&old_arc.nextstate],
                    old_arc.ilabel,
                    old_arc.olabel,
                    old_arc.weight.clone(),
                )
            }
        }

        mapping_states
    }
}

pub trait MutableArcIterator<'a>: CoreFst
where
    Self::W: 'a,
{
    type IterMut: Iterator<Item = &'a mut Arc<Self::W>>;
    fn arcs_iter_mut(&'a mut self, &StateId) -> Self::IterMut;
}
