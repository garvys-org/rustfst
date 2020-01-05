use crate::fst_traits::{Fst, StateIterator};
use crate::semirings::Semiring;
use crate::StateId;

#[derive(Debug)]
pub struct FinalState<'f, W: Semiring> {
    pub state_id: StateId,
    pub final_weight: &'f W,
}

/// Trait to iterate over the final states of a wFST.
pub trait FinalStatesIterator<'f> {
    type W: Semiring + 'f;
    type Iter: Iterator<Item = FinalState<'f, Self::W>>;
    fn final_states_iter(&'f self) -> Self::Iter;
}

impl<'f, F> FinalStatesIterator<'f> for F
where
    F: 'f + Fst,
{
    type W = F::W;
    type Iter = StructFinalStatesIterator<'f, F>;
    fn final_states_iter(&'f self) -> Self::Iter {
        StructFinalStatesIterator::new(&self)
    }
}

pub struct StructFinalStatesIterator<'f, F>
where
    F: 'f + Fst,
{
    fst: &'f F,
    it: <F as StateIterator<'f>>::Iter,
}

impl<'f, F> StructFinalStatesIterator<'f, F>
where
    F: 'f + Fst,
{
    fn new(fst: &'f F) -> StructFinalStatesIterator<F> {
        StructFinalStatesIterator {
            fst,
            it: fst.states_iter(),
        }
    }
}

impl<'f, F> Iterator for StructFinalStatesIterator<'f, F>
where
    F: 'f + Fst,
{
    type Item = FinalState<'f, F::W>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(state_id) = self.it.next() {
            if let Some(final_weight) = unsafe { self.fst.final_weight_unchecked(state_id) } {
                return Some(FinalState {
                    state_id,
                    final_weight: &final_weight,
                });
            }
        }
        None
    }
}
