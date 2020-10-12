use crate::algorithms::encode::EncodeType;
use crate::algorithms::encode::EncodeType::*;
use crate::algorithms::*;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{AllocableFst, MutableFst};
use crate::semirings::{SemiringProperties, WeaklyDivisibleSemiring, WeightQuantize};
use crate::Semiring;
use anyhow::Result;

pub fn optimize<
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
    F: MutableFst<W> + AllocableFst<W>,
>(
    fst: &mut F,
) -> Result<()>
where
    W::ReverseWeight: WeightQuantize,
{
    if fst.properties().contains(FstProperties::ACCEPTOR) {
        optimize_acceptor(fst)
    } else {
        optimize_transducer(fst)
    }
}

fn determinize<
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
    F: MutableFst<W> + AllocableFst<W>,
>(
    fst: &mut F,
) -> Result<()> {
    *fst = determinize::determinize(fst)?;
    Ok(())
}

fn encode_deter_mini_decode<
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
    F: MutableFst<W> + AllocableFst<W>,
>(
    fst: &mut F,
    encoder: EncodeType,
) -> Result<()>
where
    W::ReverseWeight: WeightQuantize,
{
    let table = encode::encode(fst, encoder)?;
    determinize(fst)?;
    minimize(fst)?;
    encode::decode(fst, table)
}

fn optimize_transducer<
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
    F: MutableFst<W> + AllocableFst<W>,
>(
    fst: &mut F,
) -> Result<()>
where
    W::ReverseWeight: WeightQuantize,
{
    if !fst.properties().contains(FstProperties::NO_EPSILONS) {
        rm_epsilon::rm_epsilon(fst)?;
    }

    tr_sum(fst);

    if !W::properties().contains(SemiringProperties::IDEMPOTENT) {
        if !fst.properties().contains(FstProperties::I_DETERMINISTIC) {
            if fst.properties().contains(FstProperties::ACYCLIC) {
                encode_deter_mini_decode(fst, EncodeLabels)?;
            }
        } else {
            minimize(fst)?;
        }
    } else {
        if !fst.properties().contains(FstProperties::I_DETERMINISTIC) {
            if !fst.properties().intersects(
                FstProperties::ACYCLIC
                    | FstProperties::UNWEIGHTED
                    | FstProperties::UNWEIGHTED_CYCLES,
            ) {
                encode_deter_mini_decode(fst, EncodeWeightsAndLabels)?;
                tr_sum(fst);
            } else {
                encode_deter_mini_decode(fst, EncodeLabels)?;
            }
        } else {
            minimize(fst)?;
        }
    }
    Ok(())
}

fn optimize_acceptor<
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
    F: MutableFst<W> + AllocableFst<W>,
>(
    fst: &mut F,
) -> Result<()>
where
    W::ReverseWeight: WeightQuantize,
{
    if !fst.properties().contains(FstProperties::NO_EPSILONS) {
        rm_epsilon::rm_epsilon(fst)?;
    }
    tr_sum(fst);
    if !W::properties().contains(SemiringProperties::IDEMPOTENT) {
        if !fst.properties().contains(FstProperties::I_DETERMINISTIC) {
            if fst.properties().contains(FstProperties::ACYCLIC) {
                determinize(fst)?;
                minimize(fst)?;
            }
        } else {
            minimize(fst)?;
        }
    } else {
        if !fst.properties().contains(FstProperties::I_DETERMINISTIC) {
            if !fst.properties().intersects(
                FstProperties::ACYCLIC
                    | FstProperties::UNWEIGHTED
                    | FstProperties::UNWEIGHTED_CYCLES,
            ) {
                encode_deter_mini_decode(fst, EncodeWeights)?;
                tr_sum(fst)
            } else {
                determinize(fst)?;
                minimize(fst)?;
            }
        } else {
            minimize(fst)?;
        }
    }
    Ok(())
}
