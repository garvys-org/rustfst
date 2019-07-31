use std::ops::Range;

use crate::fst_impls::ConstFst;
use crate::fst_traits::StateIterator;
use crate::semirings::Semiring;
use crate::StateId;

impl<'a, W: 'a + Semiring> StateIterator<'a> for ConstFst<W> {
    type Iter = Range<StateId>;
    fn states_iter(&'a self) -> Self::Iter {
        (0..self.states.len())
    }
}
