use semirings::{Semiring, WeaklyDivisibleSemiring};
use std::i32;
use std::ops::{Add, AddAssign, Mul, MulAssign};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct IntegerWeight {
    value: i32,
}

impl IntegerWeight {
    pub fn new(value: i32) -> Self {
        IntegerWeight { value }
    }
}

impl Semiring for IntegerWeight {
    type Type = i32;

    fn plus(&self, rhs: &Self) -> Self {
        Self::new(self.value + rhs.value)
    }
    fn times(&self, rhs: &Self) -> Self {
        Self::new(self.value * rhs.value)
    }

    fn zero() -> Self {
        Self::new(0)
    }

    fn one() -> Self {
        Self::new(1)
    }

    fn value(&self) -> Self::Type {
        self.value
    }

    fn set_value(&mut self, value: <Self as Semiring>::Type) {
        self.value = value
    }
}

add_mul_semiring!(IntegerWeight);
