use bitflags::bitflags;

use crate::{Label, KDELTA};

bitflags! {
    /// What kind of weight should be factored ? Tr weight ? Final weights ?
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct FactorWeightType: u32 {
        /// Factor weights located on the Trs.
        const FACTOR_FINAL_WEIGHTS = 0b01;
        /// Factor weights located in the final states.
        const FACTOR_ARC_WEIGHTS = 0b10;
    }
}

#[cfg(test)]
impl FactorWeightType {
    pub fn from_bools(factor_final_weights: bool, factor_tr_weights: bool) -> FactorWeightType {
        match (factor_final_weights, factor_tr_weights) {
            (true, true) => {
                FactorWeightType::FACTOR_FINAL_WEIGHTS | FactorWeightType::FACTOR_ARC_WEIGHTS
            }
            (true, false) => FactorWeightType::FACTOR_FINAL_WEIGHTS,
            (false, true) => FactorWeightType::FACTOR_ARC_WEIGHTS,
            (false, false) => Self::empty(),
        }
    }
}

/// Configuration to control the behaviour of the `factor_weight` algorithm.
#[derive(Clone, Debug, PartialEq)]
pub struct FactorWeightOptions {
    /// Quantization delta
    pub delta: f32,
    /// Factor transition weights and/or final weights
    pub mode: FactorWeightType,
    /// Input label of transition when factoring final weights.
    pub final_ilabel: Label,
    /// Output label of transition when factoring final weights.
    pub final_olabel: Label,
    /// When factoring final w' results in > 1 trs at state, increments ilabels to make distinct ?
    pub increment_final_ilabel: bool,
    /// When factoring final w' results in > 1 trs at state, increments olabels to make distinct ?
    pub increment_final_olabel: bool,
}

impl FactorWeightOptions {
    #[allow(unused)]
    pub fn new(mode: FactorWeightType) -> FactorWeightOptions {
        FactorWeightOptions {
            delta: KDELTA,
            mode,
            final_ilabel: 0,
            final_olabel: 0,
            increment_final_ilabel: false,
            increment_final_olabel: false,
        }
    }
}
