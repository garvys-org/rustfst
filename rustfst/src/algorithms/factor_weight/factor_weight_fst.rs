use crate::algorithms::factor_weight::factor_weight_impl::FactorWeightImpl;
use crate::algorithms::lazy_fst::LazyFst;

/// The result of weight factoring is a transducer equivalent to the
/// input whose path weights have been factored according to the FactorIterator.
/// States and transitions will be added as necessary. The algorithm is a
/// generalization to arbitrary weights of the second step of the input
/// epsilon-normalization algorithm. This version is a Delayed FST.
pub type FactorWeightFst<W, F, B, FI> = LazyFst<FactorWeightImpl<W, F, B, FI>>;

// impl<'a, W, F: Fst<W>, B: Borrow<F>, FI: FactorIterator<W>> FactorWeightFst<W, F, B, FI>
// where
//     W: WeightQuantize,
// {
//     pub fn new(fst: B, opts: FactorWeightOptions) -> Result<Self> {
//         let isymt = fst.borrow().input_symbols().cloned();
//         let osymt = fst.borrow().output_symbols().cloned();
//         Ok(Self::from_impl(
//             FactorWeightImpl::new(fst, opts)?,
//             isymt,
//             osymt,
//         ))
//     }
// }
