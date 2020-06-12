use std::borrow::Borrow;

use anyhow::Result;

use crate::algorithms::factor_weight::{FactorIterator, FactorWeightFst, FactorWeightOptions};
use crate::fst_traits::{AllocableFst, Fst, MutableFst};
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
    F2: MutableFst<W> + AllocableFst<W>,
    FI: FactorIterator<W>,
    W: WeightQuantize,
{
    let fst = FactorWeightFst::<_, _, _, FI>::new(fst_in, opts)?;
    fst.compute()
}
