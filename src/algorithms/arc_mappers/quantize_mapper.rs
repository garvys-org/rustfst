use crate::algorithms::ArcMapper;
use crate::semirings::{Semiring, WeightQuantize};
use crate::Arc;
use crate::KDELTA;

/// Mapper to quantize all weights.
pub struct QuantizeMapper {}

impl<S: WeightQuantize + Semiring> ArcMapper<S> for QuantizeMapper {
    fn arc_map(&mut self, arc: &mut Arc<S>) {
        self.final_weight_map(&mut arc.weight);
    }

    fn final_weight_map(&mut self, weight: &mut S) {
        weight.quantize_assign(KDELTA);
    }
}
