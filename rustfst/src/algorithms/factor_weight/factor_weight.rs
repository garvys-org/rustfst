use std::borrow::Borrow;

use anyhow::Result;

use crate::algorithms::cache::FstImpl;
use crate::algorithms::factor_weight::factor_weight_impl::FactorWeightImpl;
use crate::algorithms::factor_weight::{FactorIterator, FactorWeightOptions};
use crate::fst_traits::{Fst, MutableFst};
use crate::semirings::WeightQuantize;

/// The result of weight factoring is a transducer equivalent to the
/// input whose path weights have been factored according to the FactorIterator.
/// States and transitions will be added as necessary. The algorithm is a
/// generalization to arbitrary weights of the second step of the input
/// epsilon-normalization algorithm.
pub fn factor_weight<W, F1, B, F2, FI>(fst_in: B, opts: FactorWeightOptions) -> Result<F2>
where
    F1: Fst<W>,
    B: Borrow<F1>,
    F2: MutableFst<W>,
    FI: FactorIterator<W>,
    W: WeightQuantize,
{
    let mut factor_weight_impl: FactorWeightImpl<W, F1, B, FI> =
        FactorWeightImpl::new(fst_in, opts)?;
    factor_weight_impl.compute()
}
