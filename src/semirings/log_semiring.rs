use semirings::{Semiring, WeaklyDivisibleSemiring, CompleteSemiring, StarSemiring};
use std::f32;
use std::ops::{Add, AddAssign, Mul, MulAssign};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct LogWeight {
    value: f32,
}

impl LogWeight {
    pub fn new(value: f32) -> Self {
        LogWeight { value }
    }
}

fn ln_pos_exp(x: f32) -> f32 {
    ((-x).exp()).ln_1p()
}


impl Semiring for LogWeight {
    type Type = f32;

    fn plus(&self, rhs: &Self) -> Self {
        let f1 = self.value();
        let f2 = rhs.value();
        if f1 == f32::INFINITY {
            return rhs.clone();
        } else if f2 == f32::INFINITY {
            return self.clone();
        } else if f1 > f2 {
            return Self::new(f2 - ln_pos_exp(f1 - f2));
        } else {
            return Self::new(f1 - ln_pos_exp(f2 - f1));
        }
    }

    fn times(&self, rhs: &Self) -> Self {
        let f1 = self.value();
        let f2 = rhs.value();
        if f1 == f32::INFINITY {
            return self.clone();
        } else if f2 == f32::INFINITY {
            return rhs.clone();
        }
        else {
            return Self::new(f1 + f2)
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

add_mul_semiring!(LogWeight);
display_semiring!(LogWeight);

impl CompleteSemiring for LogWeight {}

impl StarSemiring for LogWeight {
    fn closure(&self) -> Self {
        if self.value >= 0.0 && self.value < 1.0 {
            Self::new((1.0 - self.value).ln())
        }
        else {
            Self::new(f32::NEG_INFINITY)
        }
    }
}

impl WeaklyDivisibleSemiring for LogWeight {
    fn inverse(&self) -> Self {
        Self::new(-self.value)
    }

    fn divide(&self, rhs: &Self) -> Self {
        Self::new(self.value - rhs.value)
    }
}
