use semirings::{Semiring, WeaklyDivisibleSemiring, CompleteSemiring, StarSemiring};
use std::f32;
use std::ops::{Add, AddAssign, Mul, MulAssign};

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
    type Type = f32;

    fn plus(&self, rhs: &Self) -> Self {
        if self.value < rhs.value {
            Self::new(self.value)
        } else {
            Self::new(rhs.value)
        }
    }

    fn times(&self, rhs: &Self) -> Self {
        let f1 = self.value();
        let f2 = rhs.value();
        if f1 == f32::INFINITY {
            return self.clone();
        } else if f2 == f32::INFINITY {
            return rhs.clone();
        } else {
            return Self::new(f1 + f2);
        }
    }

    fn zero() -> Self {
        Self::new(f32::INFINITY)
    }

    fn one() -> Self {
        Self::new(0.0)
    }

    fn value(&self) -> Self::Type {
        self.value
    }

    fn set_value(&mut self, value: <Self as Semiring>::Type) {
        self.value = value
    }
}

add_mul_semiring!(TropicalWeight);
display_semiring!(TropicalWeight);

impl CompleteSemiring for TropicalWeight {}

impl StarSemiring for TropicalWeight {
    fn closure(&self) -> Self {
        if self.value.is_sign_positive() && self.value.is_finite() {
            Self::new(0.0)
        }
        else {
            Self::new(f32::NEG_INFINITY)
        }
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
