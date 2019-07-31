use std::ops::Range;

use crate::fst_impls::VectorFst;
use crate::fst_traits::StateIterator;
use crate::semirings::Semiring;
use crate::StateId;

impl<'a, W: 'a + Semiring> StateIterator<'a> for VectorFst<W> {
    type Iter = Range<StateId>;
    fn states_iter(&'a self) -> Self::Iter {
        (0..self.states.len())
    }
}
