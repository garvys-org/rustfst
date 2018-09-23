use semirings::{Semiring, WeaklyDivisibleSemiring};
use std::f32;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct TropicalWeight {
    value: f32,
}

impl TropicalWeight {
    pub fn new(value: f32) -> Self {
        TropicalWeight { value }
    }
}

impl Semiring for TropicalWeight {
    fn plus(&self, rhs: &Self) -> Self {
        Self::new(match self.value < rhs.value {
        	true => self.value,
        	false => rhs.value,
        })
    }
    fn times(&self, rhs: &Self) -> Self {
        Self::new(self.value + rhs.value)
    }

    fn zero() -> Self {
        Self::new(f32::INFINITY)
    }

    fn one() -> Self {
        Self::new(0.0)
    }
}

impl WeaklyDivisibleSemiring for TropicalWeight {
    fn inverse(&self) -> Self {
        Self::new(-self.value)
    }

    fn divide(&self, rhs: &Self) -> Self {
        Self::new(self.value - rhs.value)
    }
}

