use arc::Arc;
use semirings::Semiring;
use Label;
use StateId;

pub trait Fst<W: Semiring> : PartialEq + for<'a> ArcIterator<'a, W> {
    //type Symtab: IntoIterator<Item=String>;
    fn start(&self) -> Option<StateId>;
    fn final_weight(&self, &StateId) -> Option<W>;
    //fn get_isyms(&self) -> Option<Self::Symtab>;
    //fn get_osyms(&self) -> Option<Self::Symtab>;
    fn is_final(&self, &StateId) -> bool;
    fn num_arcs(&self) -> usize;
}

pub trait ArcIterator<'a, W: 'a + Semiring> {
    type Iter: Iterator<Item = &'a Arc<W>>;
    fn arcs_iter(&'a self, &StateId) -> Self::Iter;
}

pub trait MutableFst<W: Semiring>: Fst<W> + for<'a> MutableArcIterator<'a, W> {
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
        weight: W,
    );
    fn set_final(&mut self, id: &StateId, finalweight: W);
    // fn set_isyms<T: IntoIterator<Item=String>>(&mut self, symtab: T);
    // fn set_osyms<T: IntoIterator<Item=String>>(&mut self, symtab: T);
}

pub trait MutableArcIterator<'a, W: 'a + Semiring> {
    type IterMut: Iterator<Item = &'a mut Arc<W>>;
    fn arcs_iter_mut(&'a mut self, &StateId) -> Self::IterMut;
}

pub trait ExpandedFst<W: Semiring>: Fst<W> {
    fn num_states(&self) -> usize;
}

// use std::cmp;
// pub fn transducer<'a, T: Iterator<Item = Label>, W: 'a + Semiring, F: MutableFst<'a, W>>(labels_input: T, labels_output: T) -> F {
//     let mut vec_labels_input: Vec<_> = labels_input.collect();
//     let mut vec_labels_output: Vec<_> = labels_output.collect();

//     let max_size = cmp::max(vec_labels_input.len(), vec_labels_output.len());

//     vec_labels_input.resize(max_size, 0);
//     vec_labels_output.resize(max_size, 0);

//     let mut fst = F::new();
//     let mut state_cour = fst.add_state();
//     fst.set_start(&state_cour);

//     for (i, o) in vec_labels_input.iter().zip(vec_labels_output.iter()) {
//         let new_state = fst.add_state();
//         fst.add_arc(&state_cour, &new_state, *i, *o, W::zero());
//         state_cour = new_state;
//     }

//     fst.set_final(&state_cour, W::one());

//     fst
// }