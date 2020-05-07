use crate::fst_impls::VectorFst;
use crate::fst_traits::ExpandedFst;
use crate::semirings::Semiring;

impl<W: 'static + Semiring> ExpandedFst<W> for VectorFst<W> {
    fn num_states(&self) -> usize {
        self.states.len()
    }
}
