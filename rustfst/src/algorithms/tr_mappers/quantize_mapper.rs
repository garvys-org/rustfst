use anyhow::Result;

use crate::algorithms::{FinalTr, MapFinalAction, TrMapper, WeightConverter};
use crate::fst_properties::FstProperties;
use crate::semirings::{Semiring, WeightQuantize};
use crate::Tr;
use crate::KDELTA;

/// Mapper to quantize all weights.
#[derive(Debug, Copy, Clone)]
pub struct QuantizeMapper {
    delta: f32,
}

impl QuantizeMapper {
    pub fn new(delta: f32) -> Self {
        Self { delta }
    }
}

impl Default for QuantizeMapper {
    fn default() -> Self {
        Self { delta: KDELTA }
    }
}

impl<S: WeightQuantize + Semiring> TrMapper<S> for QuantizeMapper {
    fn tr_map(&self, tr: &mut Tr<S>) -> Result<()> {
        tr.weight.quantize_assign(self.delta)
    }

    fn final_tr_map(&self, final_tr: &mut FinalTr<S>) -> Result<()> {
        final_tr.weight.quantize_assign(self.delta)
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }

    fn properties(&self, inprops: FstProperties) -> FstProperties {
        inprops & FstProperties::weight_invariant_properties()
    }
}

impl<S> WeightConverter<S, S> for QuantizeMapper
where
    S: WeightQuantize,
{
    tr_mapper_to_weight_convert_mapper_methods!(S);
}
