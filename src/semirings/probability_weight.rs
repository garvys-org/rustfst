use std::f32;
use std::hash::{Hash, Hasher};

use failure::Fallible;

use ordered_float::OrderedFloat;

use crate::semirings::{
    CompleteSemiring, DivideType, Semiring, SemiringProperties, StarSemiring,
    WeaklyDivisibleSemiring, WeightQuantize,
};
use crate::KDELTA;

/// Probability semiring: (x, +, 0.0, 1.0).
#[derive(Clone, Debug, PartialOrd, Default, Copy, Eq)]
pub struct ProbabilityWeight {
    value: OrderedFloat<f32>,
}

impl Semiring for ProbabilityWeight {
    type Type = f32;

    fn zero() -> Self {
        Self {
            value: OrderedFloat(0.0),
        }
    }
    fn one() -> Self {
        Self {
            value: OrderedFloat(1.0),
        }
    }

    fn new(value: <Self as Semiring>::Type) -> Self {
        ProbabilityWeight {
            value: OrderedFloat(value),
        }
    }

    fn plus_assign<P: AsRef<Self>>(&mut self, rhs: P) -> Fallible<()> {
        self.value.0 += rhs.as_ref().value.0;
        Ok(())
    }

    fn times_assign<P: AsRef<Self>>(&mut self, rhs: P) -> Fallible<()> {
        self.value.0 *= rhs.as_ref().value.0;
        Ok(())
    }

    fn value(&self) -> Self::Type {
        self.value.into_inner()
    }

    fn set_value(&mut self, value: <Self as Semiring>::Type) {
        self.value.0 = value
    }

    fn properties() -> SemiringProperties {
        SemiringProperties::LEFT_SEMIRING
            | SemiringProperties::RIGHT_SEMIRING
            | SemiringProperties::COMMUTATIVE
    }
}

impl AsRef<ProbabilityWeight> for ProbabilityWeight {
    fn as_ref(&self) -> &ProbabilityWeight {
        &self
    }
}

display_semiring!(ProbabilityWeight);

impl CompleteSemiring for ProbabilityWeight {}

impl StarSemiring for ProbabilityWeight {
    fn closure(&self) -> Self {
        Self::new(1.0 / (1.0 - self.value.0))
    }
}

impl WeaklyDivisibleSemiring for ProbabilityWeight {
    fn divide(&self, rhs: &Self, _divide_type: DivideType) -> Fallible<Self> {
        // May panic if rhs.value == 0.0
        if rhs.value.0 == 0.0 {
            bail!("Division bby 0")
        }
        Ok(Self::new(self.value.0 / rhs.value.0))
    }
}

impl_quantize_f32!(ProbabilityWeight);

partial_eq_and_hash_f32!(ProbabilityWeight);
