use std::f32;

use crate::semirings::{
    CompleteSemiring, Semiring, StarSemiring, WeaklyDivisibleSemiring, WeightQuantize,
};
use crate::KDELTA;

#[derive(Clone, Debug, PartialOrd, Default, Copy)]
pub struct LogWeight {
    value: f32,
}

fn ln_pos_exp(x: f32) -> f32 {
    ((-x).exp()).ln_1p()
}

impl Semiring for LogWeight {
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
        LogWeight { value }
    }

    fn plus_assign<P: AsRef<Self>>(&mut self, rhs: P) {
        let f1 = self.value();
        let f2 = rhs.as_ref().value();
        self.value = if f1 == f32::INFINITY {
            f2
        } else if f2 == f32::INFINITY {
            f1
        } else if f1 > f2 {
            f2 - ln_pos_exp(f1 - f2)
        } else {
            f1 - ln_pos_exp(f2 - f1)
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

impl AsRef<LogWeight> for LogWeight {
    fn as_ref(&self) -> &LogWeight {
        &self
    }
}

display_semiring!(LogWeight);

impl CompleteSemiring for LogWeight {}

impl StarSemiring for LogWeight {
    fn closure(&self) -> Self {
        if self.value >= 0.0 && self.value < 1.0 {
            Self::new((1.0 - self.value).ln())
        } else {
            Self::new(f32::NEG_INFINITY)
        }
    }
}

impl WeaklyDivisibleSemiring for LogWeight {
    fn inverse_mut(&mut self) {
        self.value = -self.value;
    }

    fn divide(&self, rhs: &Self) -> Self {
        Self::new(self.value - rhs.value)
    }
}

impl WeightQuantize for LogWeight {}

partial_eq_f32!(LogWeight);
