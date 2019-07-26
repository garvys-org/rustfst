use std::i32;

use failure::Fallible;

use crate::semirings::{CompleteSemiring, Semiring, SemiringProperties, StarSemiring};

/// Probability semiring: (x, +, 0.0, 1.0).
#[derive(Clone, Debug, PartialEq, PartialOrd, Default, Hash, Eq, Copy)]
pub struct IntegerWeight {
    value: i32,
}

impl Semiring for IntegerWeight {
    type Type = i32;
    type ReverseWeight = IntegerWeight;

    fn zero() -> Self {
        Self { value: 0 }
    }
    fn one() -> Self {
        Self { value: 1 }
    }

    fn new(value: <Self as Semiring>::Type) -> Self {
        IntegerWeight { value }
    }

    fn plus_assign<P: AsRef<Self>>(&mut self, rhs: P) -> Fallible<()> {
        self.value += rhs.as_ref().value;
        Ok(())
    }

    fn times_assign<P: AsRef<Self>>(&mut self, rhs: P) -> Fallible<()> {
        self.value *= rhs.as_ref().value;
        Ok(())
    }

    fn value(&self) -> &Self::Type {
        &self.value
    }

    fn take_value(self) -> Self::Type {
        self.value
    }

    fn set_value(&mut self, value: <Self as Semiring>::Type) {
        self.value = value
    }

    fn reverse(&self) -> Fallible<Self::ReverseWeight> {
        Ok(*self)
    }

    fn properties() -> SemiringProperties {
        SemiringProperties::LEFT_SEMIRING
            | SemiringProperties::RIGHT_SEMIRING
            | SemiringProperties::COMMUTATIVE
    }
}

impl AsRef<IntegerWeight> for IntegerWeight {
    fn as_ref(&self) -> &IntegerWeight {
        &self
    }
}

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
