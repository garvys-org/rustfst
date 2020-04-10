use anyhow::Result;

use crate::algorithms::{ArcMapper, FinalArc, MapFinalAction, WeightConverter};
use crate::semirings::Semiring;
use crate::Arc;

/// Mapper to map all non-Zero() weights to One().
pub struct RmWeightMapper {}

pub fn map_weight<W: Semiring>(weight: &mut W) {
    if !weight.is_zero() {
        weight.set_value(W::one().take_value())
    }
}

impl<S: Semiring> ArcMapper<S> for RmWeightMapper {
    fn arc_map(&self, arc: &mut Arc<S>) -> Result<()> {
        map_weight(&mut arc.weight);
        Ok(())
    }

    fn final_arc_map(&self, final_arc: &mut FinalArc<S>) -> Result<()> {
        map_weight(&mut final_arc.weight);
        Ok(())
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}

arc_mapper_to_weight_convert_mapper!(RmWeightMapper);
