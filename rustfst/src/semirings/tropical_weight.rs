use std::f32;
use std::hash::{Hash, Hasher};

use failure::Fallible;

use crate::semirings::{
    CompleteSemiring, DivideType, Semiring, SemiringProperties, StarSemiring,
    WeaklyDivisibleSemiring, WeightQuantize,
};
use crate::KDELTA;

use ordered_float::OrderedFloat;

/// Tropical semiring: (min, +, inf, 0).
#[derive(Clone, Debug, PartialOrd, Default, Copy, Eq)]
pub struct TropicalWeight {
    value: OrderedFloat<f32>,
}

impl Semiring for TropicalWeight {
    type Type = f32;
    type ReverseWeight = TropicalWeight;

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
        TropicalWeight {
            value: OrderedFloat(value),
        }
    }

    fn plus_assign<P: AsRef<Self>>(&mut self, rhs: P) -> Fallible<()> {
        if rhs.as_ref().value < self.value {
            self.value = rhs.as_ref().value;
        }
        Ok(())
    }

    fn times_assign<P: AsRef<Self>>(&mut self, rhs: P) -> Fallible<()> {
        let f1 = self.value();
        let f2 = rhs.as_ref().value();
        if f1 == &f32::INFINITY {
        } else if f2 == &f32::INFINITY {
            self.value.0 = *f2;
        } else {
            self.value.0 += f2;
        }
        Ok(())
    }

    fn value(&self) -> &Self::Type {
        &self.value.0
    }

    fn take_value(self) -> Self::Type {
        self.value.0
    }

    fn set_value(&mut self, value: <Self as Semiring>::Type) {
        self.value.0 = value
    }

    fn reverse(&self) -> Fallible<Self::ReverseWeight> {
        Ok(*self)
    }

    fn properties() -> SemiringProperties {
        SemiringProperties::LEFT_SEMIRING
            | SemiringProperties::RIGHT_SEMIRING
            | SemiringProperties::COMMUTATIVE
            | SemiringProperties::PATH
            | SemiringProperties::IDEMPOTENT
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
    fn divide_assign(&mut self, rhs: &Self, _divide_type: DivideType) -> Fallible<()> {
        self.value.0 -= rhs.value.0;
        Ok(())
    }
}

impl_quantize_f32!(TropicalWeight);

partial_eq_and_hash_f32!(TropicalWeight);
