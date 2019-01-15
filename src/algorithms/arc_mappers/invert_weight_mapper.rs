use crate::algorithms::ArcMapper;
use crate::semirings::WeaklyDivisibleSemiring;
use crate::Arc;

/// Mapper to reciprocate all non-Zero() weights.
pub struct InvertWeightMapper {}

impl<S: WeaklyDivisibleSemiring> ArcMapper<S> for InvertWeightMapper {
    fn arc_map(&mut self, arc: &mut Arc<S>) {
        self.final_weight_map(&mut arc.weight);
    }

    fn final_weight_map(&mut self, weight: &mut S) {
        weight.inverse_assign();
    }
}
