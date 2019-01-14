use crate::semirings::{
    CompleteSemiring, Semiring, StarSemiring, WeaklyDivisibleSemiring, WeightQuantize,
};
use crate::KDELTA;

#[derive(Clone, Debug, PartialOrd, Default, Copy)]
pub struct ProbabilityWeight {
    value: f32,
}

impl Semiring for ProbabilityWeight {
    type Type = f32;

    fn zero() -> Self {
        Self { value: 0.0 }
    }
    fn one() -> Self {
        Self { value: 1.0 }
    }

    fn new(value: <Self as Semiring>::Type) -> Self {
        ProbabilityWeight { value }
    }

    fn plus_assign<P: AsRef<Self>>(&mut self, rhs: P) {
        self.value += rhs.as_ref().value;
    }

    fn times_assign<P: AsRef<Self>>(&mut self, rhs: P) {
        self.value *= rhs.as_ref().value;
    }

    fn value(&self) -> Self::Type {
        self.value
    }

    fn set_value(&mut self, value: <Self as Semiring>::Type) {
        self.value = value
    }
}

impl AsRef<ProbabilityWeight> for ProbabilityWeight {
    fn as_ref(&self) -> &ProbabilityWeight {
        &self
    }
}

display_semiring!(ProbabilityWeight);

impl CompleteSemiring for ProbabilityWeight {}

impl StarSemiring for ProbabilityWeight {
    fn closure(&self) -> Self {
        Self::new(1.0 / (1.0 - self.value))
    }
}

impl WeaklyDivisibleSemiring for ProbabilityWeight {
    fn inverse_mut(&mut self) {
        // May panic if self.value == 0
        self.value = 1.0 / self.value;
    }

    fn divide(&self, rhs: &Self) -> Self {
        // May panic if rhs.value == 0.0
        Self::new(self.value / rhs.value)
    }
}

impl WeightQuantize for ProbabilityWeight {}

partial_eq_f32!(ProbabilityWeight);
