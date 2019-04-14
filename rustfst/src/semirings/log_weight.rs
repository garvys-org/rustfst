use std::f32;
use std::hash::{Hash, Hasher};

use failure::Fallible;

use ordered_float::OrderedFloat;

use crate::semirings::{
    CompleteSemiring, DivideType, Semiring, SemiringProperties, StarSemiring,
    WeaklyDivisibleSemiring, WeightQuantize,
};
use crate::KDELTA;

/// Log semiring: (log(e^-x + e^-y), +, inf, 0).
#[derive(Clone, Debug, PartialOrd, Default, Copy, Eq)]
pub struct LogWeight {
    value: OrderedFloat<f32>,
}

fn ln_pos_exp(x: f32) -> f32 {
    ((-x).exp()).ln_1p()
}

impl Semiring for LogWeight {
    type Type = f32;
    type ReverseSemiring = LogWeight;

    fn zero() -> Self {
        Self {
            value: OrderedFloat(f32::INFINITY),
        }
    }
    fn one() -> Self {
        Self {
            value: OrderedFloat(0.0),
        }
    }

    fn new(value: <Self as Semiring>::Type) -> Self {
        LogWeight {
            value: OrderedFloat(value),
        }
    }

    fn plus_assign<P: AsRef<Self>>(&mut self, rhs: P) -> Fallible<()> {
        let f1 = self.value();
        let f2 = rhs.as_ref().value();
        self.value.0 = if f1 == f32::INFINITY {
            f2
        } else if f2 == f32::INFINITY {
            f1
        } else if f1 > f2 {
            f2 - ln_pos_exp(f1 - f2)
        } else {
            f1 - ln_pos_exp(f2 - f1)
        };
        Ok(())
    }

    fn times_assign<P: AsRef<Self>>(&mut self, rhs: P) -> Fallible<()> {
        let f1 = self.value();
        let f2 = rhs.as_ref().value();
        if f1 == f32::INFINITY {
        } else if f2 == f32::INFINITY {
            self.value.0 = f2;
        } else {
            self.value.0 += f2;
        }
        Ok(())
    }

    fn value(&self) -> Self::Type {
        self.value.0
    }

    fn set_value(&mut self, value: <Self as Semiring>::Type) {
        self.value.0 = value
    }

    fn reverse(&self) -> Fallible<Self::ReverseSemiring> {
        Ok(self.clone())
    }

    fn properties() -> SemiringProperties {
        SemiringProperties::LEFT_SEMIRING
            | SemiringProperties::RIGHT_SEMIRING
            | SemiringProperties::COMMUTATIVE
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
        if self.value.0 >= 0.0 && self.value.0 < 1.0 {
            Self::new((1.0 - self.value.0).ln())
        } else {
            Self::new(f32::NEG_INFINITY)
        }
    }
}

impl WeaklyDivisibleSemiring for LogWeight {
    fn divide(&self, rhs: &Self, _divide_type: DivideType) -> Fallible<Self> {
        Ok(Self::new(self.value.0 - rhs.value.0))
    }
}

impl_quantize_f32!(LogWeight);

partial_eq_and_hash_f32!(LogWeight);
