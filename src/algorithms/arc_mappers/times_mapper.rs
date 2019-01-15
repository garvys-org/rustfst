use crate::algorithms::ArcMapper;
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
}

impl<S: Semiring> ArcMapper<S> for TimesMapper<S> {
    fn arc_map(&mut self, arc: &mut Arc<S>) {
        self.final_weight_map(&mut arc.weight);
    }

    fn final_weight_map(&mut self, weight: &mut S) {
        weight.times_assign(&self.to_multiply);
    }
}
