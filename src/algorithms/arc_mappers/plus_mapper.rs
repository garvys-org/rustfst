use crate::algorithms::ArcMapper;
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
}

impl<S: Semiring> ArcMapper<S> for PlusMapper<S> {
    fn arc_map(&mut self, arc: &mut Arc<S>) {
        self.final_weight_map(&mut arc.weight);
    }

    fn final_weight_map(&mut self, weight: &mut S) {
        weight.plus_assign(&self.to_add);
    }
}
