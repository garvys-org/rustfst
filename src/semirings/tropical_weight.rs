use std::f32;

use crate::semirings::{
    CompleteSemiring, Semiring, StarSemiring, WeaklyDivisibleSemiring, WeightQuantize,
};
use crate::KDELTA;

#[derive(Clone, Debug, PartialOrd, Default, Copy)]
pub struct TropicalWeight {
    value: f32,
}

impl Semiring for TropicalWeight {
    type Type = f32;

    fn zero() -> Self {
        Self {
            value: f32::INFINITY,
        }
    }

    fn one() -> Self {
        Self { value: 0.0 }
    }

    fn new(value: <Self as Semiring>::Type) -> Self {
        TropicalWeight { value }
    }

    fn plus_assign<P: AsRef<Self>>(&mut self, rhs: P) {
        if rhs.as_ref().value < self.value {
            self.value = rhs.as_ref().value;
        }
    }

    fn times_assign<P: AsRef<Self>>(&mut self, rhs: P) {
        let f1 = self.value();
        let f2 = rhs.as_ref().value();
        if f1 == f32::INFINITY {
        } else if f2 == f32::INFINITY {
            self.value = f2;
        } else {
            self.value += f2;
        }
    }

    fn value(&self) -> Self::Type {
        self.value
    }

    fn set_value(&mut self, value: <Self as Semiring>::Type) {
        self.value = value
    }
}

impl AsRef<TropicalWeight> for TropicalWeight {
    fn as_ref(&self) -> &TropicalWeight {
        &self
    }
}

display_semiring!(TropicalWeight);

impl CompleteSemiring for TropicalWeight {}

impl StarSemiring for TropicalWeight {
    fn closure(&self) -> Self {
        if self.value.is_sign_positive() && self.value.is_finite() {
            Self::new(0.0)
        } else {
            Self::new(f32::NEG_INFINITY)
        }
    }
}

impl WeaklyDivisibleSemiring for TropicalWeight {
    fn inverse_mut(&mut self) {
        self.value = -self.value;
    }

    fn divide(&self, rhs: &Self) -> Self {
        Self::new(self.value - rhs.value)
    }
}

impl WeightQuantize for TropicalWeight {}

partial_eq_f32!(TropicalWeight);
