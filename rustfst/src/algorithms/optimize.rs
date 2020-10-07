use crate::algorithms::encode::EncodeType;
use crate::algorithms::encode::EncodeType::*;
use crate::algorithms::*;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{MutableFst, AllocableFst};
use crate::semirings::{SemiringProperties, WeaklyDivisibleSemiring, WeightQuantize};
use crate::Semiring;
use anyhow::Result;

pub fn optimize<W: Semiring + WeaklyDivisibleSemiring + WeightQuantize, F: MutableFst<W> + AllocableFst<W>>(fst: &mut F) -> Result<()> {
    // fst.compute_and_update_properties_all()?;
    // println!("{}", fst.properties().bits());
    if fst.properties().contains(FstProperties::ACCEPTOR) {
        println!("Optimize acceptor");
        optimize_acceptor(fst)
    } else {
        println!("Optimize transducer");
        optimize_transducer(fst)
    }
}

fn determinize<W: Semiring + WeaklyDivisibleSemiring + WeightQuantize, F: MutableFst<W> + AllocableFst<W>>(fst: &mut F) -> Result<()> {
    *fst = determinize::determinize(fst, determinize::DeterminizeType::DeterminizeFunctional)?;
    Ok(())
}

fn encode_deter_mini_decode<W: Semiring + WeaklyDivisibleSemiring + WeightQuantize, F: MutableFst<W> + AllocableFst<W>>(fst: &mut F, encoder: EncodeType) -> Result<()> {
    let table = encode::encode(fst, encoder)?;
    dbg!("encoded");
    determinize(fst)?;
    dbg!("det");
    minimize(fst, false)?;
    dbg!("mini");
    encode::decode(fst, table)
}

fn optimize_transducer<W: Semiring + WeaklyDivisibleSemiring + WeightQuantize, F: MutableFst<W> + AllocableFst<W>>(fst: &mut F) -> Result<()> {
    if !fst.properties().contains(FstProperties::NO_EPSILONS) {
        println!("RmEpsilon");
        rm_epsilon::rm_epsilon(fst)?;
    }
    // fst.compute_and_update_properties_all()?;
    println!("ArcSum");
    tr_sum(fst);
    // fst.compute_and_update_properties_all()?;
    if !W::properties().contains(SemiringProperties::IDEMPOTENT) {
        println!("W not idempotent");
        if !fst.properties().contains(FstProperties::I_DETERMINISTIC) {
            println!("Not IDeterministic");
            if fst.properties().contains(FstProperties::ACYCLIC) {
                println!("Acyclic -> encode, determinize, minimimize and decode");
                encode_deter_mini_decode(fst, EncodeLabels)?;
            }
        } else {
            println!("IDeterministic -> Minimize");
            minimize(fst, false)?;
        }
    } else {
        println!("W idempotent");
        if !fst.properties().contains(FstProperties::I_DETERMINISTIC) {
            println!("Not IDeterministic");
            if !fst.properties().intersects(
                FstProperties::ACYCLIC
                    | FstProperties::UNWEIGHTED
                    | FstProperties::UNWEIGHTED_CYCLES,
            ) {
                println!("EncodeWeights and Labels -> encode, deter, min, decode, arcsum");
                encode_deter_mini_decode(fst, EncodeWeightsAndLabels)?;
                // FIXME: Missing ArcSum no ?
                println!("TrSum");
                tr_sum(fst);
            } else {
                println!("EncodeLabels -> encode, deter, min, decode");
                encode_deter_mini_decode(fst, EncodeLabels)?;
            }
        } else {
            println!("IDeterministic -> Minimize");
            minimize(fst, false)?;
            // dbg!(fst.properties().contains(FstProperties::I_DETERMINISTIC));
        }
    }
    println!("Props output = {:?}", fst.properties());
    Ok(())
}

fn optimize_acceptor<W: Semiring + WeaklyDivisibleSemiring + WeightQuantize, F: MutableFst<W> + AllocableFst<W>>(fst: &mut F) -> Result<()> {
    if !fst.properties().contains(FstProperties::NO_EPSILONS) {
        rm_epsilon::rm_epsilon(fst)?;
    }
    tr_sum(fst);
    if !W::properties().contains(SemiringProperties::IDEMPOTENT) {
        if !fst.properties().contains(FstProperties::I_DETERMINISTIC) {
            if fst.properties().contains(FstProperties::ACYCLIC) {
                determinize(fst)?;
                minimize(fst, false)?;
            }
        } else {
            minimize(fst, false)?;
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
                minimize(fst, false)?;
            }
        } else {
            minimize(fst, false)?;
        }
    }
    Ok(())
}