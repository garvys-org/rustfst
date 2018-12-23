use std::i32;
use std::ops::{Add, AddAssign, Mul, MulAssign};

use crate::semirings::CompleteSemiring;
use crate::semirings::Semiring;
use crate::semirings::StarSemiring;

#[derive(Clone, Debug, PartialEq, Default, Hash, Eq)]
pub struct IntegerWeight {
    value: i32,
}

impl Semiring for IntegerWeight {
    type Type = i32;

    const ZERO: Self = Self { value: 0 };
    const ONE: Self = Self { value: 1 };

    fn new(value: <Self as Semiring>::Type) -> Self {
        IntegerWeight { value }
    }

    fn plus(&self, rhs: &Self) -> Self {
        Self::new(self.value + rhs.value)
    }
    fn times(&self, rhs: &Self) -> Self {
        Self::new(self.value * rhs.value)
    }

    fn value(&self) -> Self::Type {
        self.value
    }

    fn set_value(&mut self, value: <Self as Semiring>::Type) {
        self.value = value
    }
}

add_mul_semiring!(IntegerWeight);
display_semiring!(IntegerWeight);

impl CompleteSemiring for IntegerWeight {}

impl StarSemiring for IntegerWeight {
    fn closure(&self) -> Self {
        if self.value == 0 {
            return Self::new(1);
        }
        Self::new(i32::max_value())
    }
}
