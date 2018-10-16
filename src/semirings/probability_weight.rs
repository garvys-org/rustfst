use semirings::{Semiring, WeaklyDivisibleSemiring, CompleteSemiring, StarSemiring};
use std::ops::{Add, AddAssign, Mul, MulAssign};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ProbabilityWeight {
    value: f32,
}

impl ProbabilityWeight {
    pub fn new(value: f32) -> Self {
        ProbabilityWeight { value }
    }
}

impl Semiring for ProbabilityWeight {
    type Type = f32;

    fn plus(&self, rhs: &Self) -> Self {
        Self::new(self.value + rhs.value)
    }

    fn times(&self, rhs: &Self) -> Self {
        Self::new(self.value * rhs.value)
    }

    fn zero() -> Self {
        Self::new(0.0)
    }

    fn one() -> Self {
        Self::new(1.0)
    }

    fn value(&self) -> Self::Type {
        self.value
    }

    fn set_value(&mut self, value: <Self as Semiring>::Type) {
        self.value = value
    }
}

add_mul_semiring!(ProbabilityWeight);
display_semiring!(ProbabilityWeight);

impl CompleteSemiring for ProbabilityWeight {}

impl StarSemiring for ProbabilityWeight {
    fn closure(&self) -> Self {
        Self::new(1.0 / (1.0 - self.value))
    }
}

impl WeaklyDivisibleSemiring for ProbabilityWeight {
    fn inverse(&self) -> Self {
        // May panic if self.value == 0
        Self::new(1.0 / self.value)
    }

    fn divide(&self, rhs: &Self) -> Self {
        // May panic if rhs.value == 0.0
        Self::new(self.value / rhs.value)
    }
}
