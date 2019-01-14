use crate::algorithms::ArcMapper;
use crate::semirings::Semiring;
use crate::Arc;

/// Mapper to map all non-Zero() weights to One().
pub struct RmWeightMapper {}

impl<S: Semiring> ArcMapper<S> for RmWeightMapper {
    fn arc_map(&mut self, arc: &mut Arc<S>) {
        self.final_weight_map(&mut arc.weight);
    }

    fn final_weight_map(&mut self, weight: &mut S) {
        if !weight.is_zero() {
            weight.set_value(S::one().value())
        }
    }
}
