use crate::algorithms::lazy_fst::LazyFst;
use crate::algorithms::rm_epsilon::rm_epsilon_op::RmEpsilonImpl;

/// Removes epsilon-transitions (when both the input and output label are an
/// epsilon) from a transducer. The result will be an equivalent FST that has no
/// such epsilon transitions. This version is a delayed FST.
pub type RmEpsilonFst<W, F, B> = LazyFst<RmEpsilonImpl<W, F, B>>;
// impl<W: Semiring, F: MutableFst<W>, B: Borrow<F>> RmEpsilonFst<W, F, B>
// {
//     pub fn new(fst: B) -> Self {
//         let isymt = fst.borrow().input_symbols().cloned();
//         let osymt = fst.borrow().output_symbols().cloned();
//         Self::from_impl(RmEpsilonImpl::new(fst), isymt, osymt)
//     }
// }
