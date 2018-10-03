use fst_traits::{Fst, StateIterator};
use StateId;
use semirings::Semiring;

pub struct FinalState<W: Semiring> {
    pub state_id: StateId,
    pub final_weight: W,
} 

/// Trait to iterate over the final states of a wFST
pub trait FinalStatesIterator<'a> {
    type W: Semiring;
    type Iter: Iterator<Item = FinalState<Self::W>>;
    fn final_states_iter(&'a self) -> Self::Iter;
}

impl<'a, F> FinalStatesIterator<'a> for F
where
    F: 'a + Fst,
{
    type W = F::W;
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
    fn new(fst: &'a F) -> StructFinalStatesIterator<F> {
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
    type Item = FinalState<F::W>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(state_id) = self.it.next() {
            if let Some(final_weight) = self.fst.final_weight(&state_id) {
                return Some(FinalState { state_id, final_weight });
            }
        }
        None
    }
}
