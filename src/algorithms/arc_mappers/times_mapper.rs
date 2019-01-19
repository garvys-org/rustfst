use crate::algorithms::{ArcMapper, FinalArc, MapFinalAction};
use crate::semirings::Semiring;
use crate::Arc;

/// Mapper to (right) multiply a constant to all weights.
pub struct TimesMapper<W: Semiring> {
    to_multiply: W,
}

impl<W: Semiring> TimesMapper<W> {
    pub fn new(value: W::Type) -> Self {
        TimesMapper {
            to_multiply: W::new(value),
        }
    }

    pub fn map_weight(&self, weight: &mut W) {
        weight.times_assign(&self.to_multiply);
    }
}

impl<S: Semiring> ArcMapper<S> for TimesMapper<S> {
    fn arc_map(&mut self, arc: &mut Arc<S>) {
        self.map_weight(&mut arc.weight);
    }

    fn final_arc_map(&mut self, final_arc: &mut FinalArc<S>) {
        self.map_weight(&mut final_arc.weight)
    }

    fn final_action(&self) -> MapFinalAction {
        MapFinalAction::MapNoSuperfinal
    }
}
