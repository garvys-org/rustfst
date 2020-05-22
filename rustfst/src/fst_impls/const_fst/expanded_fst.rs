use crate::fst_impls::ConstFst;
use crate::fst_traits::ExpandedFst;
use crate::semirings::Semiring;

impl<W: 'static + Semiring> ExpandedFst<W> for ConstFst<W> {
    fn num_states(&self) -> usize {
        self.states.len()
    }
}
