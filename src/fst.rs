use arc::Arc;
use semirings::Semiring;
use std::collections::HashMap;
use Label;
use StateId;

pub trait CoreFst {
    type W: Semiring;
    //type Symtab: IntoIterator<Item=String>;
    fn start(&self) -> Option<StateId>;
    fn final_weight(&self, &StateId) -> Option<<Self as CoreFst>::W>;
    //fn get_isyms(&self) -> Option<Self::Symtab>;
    //fn get_osyms(&self) -> Option<Self::Symtab>;
    fn is_final(&self, &StateId) -> bool;
    fn num_arcs(&self) -> usize;
}

pub trait Fst: CoreFst + PartialEq + for<'a> ArcIterator<'a> + for<'b> StateIterator<'b> {}

pub trait StateIterator<'a> {
    type Iter: Iterator<Item = StateId>;
    fn states_iter(&'a self) -> Self::Iter;
}

pub trait FinalStateIterator<'a> {
    type Iter: Iterator<Item = StateId>;
    fn final_states_iter(&'a self) -> Self::Iter;
}

impl<'a, F> FinalStateIterator<'a> for F
where
    F: 'a + Fst,
{
    type Iter = StructFinalStateIterator<'a, F>;
    fn final_states_iter(&'a self) -> Self::Iter {
        StructFinalStateIterator::new(&self)
    }
}

// use std::marker::PhantomData;
pub struct StructFinalStateIterator<'a, F>
where
    F: 'a + Fst,
{
    fst: &'a F,
    it: <F as StateIterator<'a>>::Iter,
}

impl<'a, F> StructFinalStateIterator<'a, F>
where
    F: 'a + Fst,
{
    pub fn new(fst: &'a F) -> StructFinalStateIterator<F> {
        StructFinalStateIterator {
            fst: fst,
            it: fst.states_iter(),
        }
    }
}

impl<'a, F> Iterator for StructFinalStateIterator<'a, F>
where
    F: 'a + Fst,
{
    type Item = StateId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(state_id) = self.it.next() {
            if self.fst.is_final(&state_id) {
                return Some(state_id);
            }
        }
        None
    }
}

pub trait ArcIterator<'a>: CoreFst
where
    Self::W: 'a,
{
    type Iter: Iterator<Item = &'a Arc<Self::W>>;
    fn arcs_iter(&'a self, &StateId) -> Self::Iter;
}

pub trait MutableFst: CoreFst + for<'a> MutableArcIterator<'a> {
    // type W: Semiring;
    fn new() -> Self;
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
                    mapping_states.get(&old_state_id).unwrap(),
                    mapping_states.get(&old_arc.nextstate).unwrap(),
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

pub trait ExpandedFst: Fst {
    fn num_states(&self) -> usize;
}

use std::cmp;
pub fn transducer<T: Iterator<Item = Label>, F: MutableFst>(
    labels_input: T,
    labels_output: T,
) -> F {
    let mut vec_labels_input: Vec<_> = labels_input.collect();
    let mut vec_labels_output: Vec<_> = labels_output.collect();

    let max_size = cmp::max(vec_labels_input.len(), vec_labels_output.len());

    vec_labels_input.resize(max_size, 0);
    vec_labels_output.resize(max_size, 0);

    let mut fst = F::new();
    let mut state_cour = fst.add_state();
    fst.set_start(&state_cour);

    for (i, o) in vec_labels_input.iter().zip(vec_labels_output.iter()) {
        let new_state = fst.add_state();
        fst.add_arc(&state_cour, &new_state, *i, *o, <F as CoreFst>::W::one());
        state_cour = new_state;
    }

    fst.set_final(&state_cour, <F as CoreFst>::W::one());

    fst
}
