use crate::fst_traits::{Fst, StateIterator};
use crate::{Semiring, StateId};
use std::marker::PhantomData;

pub struct FinalStatesIterator<'a, W, F>
where
    W: Semiring,
    F: Fst<W>,
{
    pub(crate) fst: &'a F,
    pub(crate) state_iter: <F as StateIterator<'a>>::Iter,
    pub(crate) w: PhantomData<W>,
}

impl<'a, W, F> Iterator for FinalStatesIterator<'a, W, F>
where
    W: Semiring,
    F: Fst<W>,
{
    type Item = StateId;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(s) = self.state_iter.next() {
                if unsafe { self.fst.is_final_unchecked(s) } {
                    return Some(s);
                }
            } else {
                return None;
            }
        }
    }
}
