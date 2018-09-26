use fst_traits::{Fst, StateIterator};
use StateId;

/// Trait to iterate over the final states of a wFST
pub trait FinalStatesIterator<'a> {
    type Iter: Iterator<Item = StateId>;
    fn final_states_iter(&'a self) -> Self::Iter;
}

impl<'a, F> FinalStatesIterator<'a> for F
where
    F: 'a + Fst,
{
    type Iter = StructFinalStatesIterator<'a, F>;
    fn final_states_iter(&'a self) -> Self::Iter {
        StructFinalStatesIterator::new(&self)
    }
}

pub struct StructFinalStatesIterator<'a, F>
where
    F: 'a + Fst,
{
    fst: &'a F,
    it: <F as StateIterator<'a>>::Iter,
}

impl<'a, F> StructFinalStatesIterator<'a, F>
where
    F: 'a + Fst,
{
    pub fn new(fst: &'a F) -> StructFinalStatesIterator<F> {
        StructFinalStatesIterator {
            fst,
            it: fst.states_iter(),
        }
    }
}

impl<'a, F> Iterator for StructFinalStatesIterator<'a, F>
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
