use failure::Fallible;

use crate::algorithms::{ArcMapper, FinalArc, MapFinalAction, WeightConverter};
use crate::semirings::Semiring;
use crate::Arc;

/// Mapper to add a constant to all weights.
pub struct PlusMapper<W: Semiring> {
    to_add: W,
}

impl<W: Semiring> PlusMapper<W> {
    pub fn new(value: W::Type) -> Self {
        PlusMapper {
            to_add: W::new(value),
        }
    }

    pub fn map_weight(&self, weight: &mut W) -> Fallible<()> {
        weight.plus_assign(&self.to_add)
    }
}

impl<S: Semiring> ArcMapper<S> for PlusMapper<S> {
    fn arc_map(&mut self, arc: &mut Arc<S>) -> Fallible<()> {
        self.map_weight(&mut arc.weight)
    }

    fn final_arc_map(&mut self, final_arc: &mut FinalArc<S>) -> Fallible<()> {
        self.map_weight(&mut final_arc.weight)
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}

arc_mapper_to_weight_convert_mapper!(PlusMapper<S>);
