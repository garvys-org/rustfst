use std::borrow::Borrow;
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
    type ReverseWeight = ProbabilityWeight;

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

    fn plus_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Fallible<()> {
        self.value.0 += rhs.borrow().value.0;
        Ok(())
    }

    fn times_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Fallible<()> {
        self.value.0 *= rhs.borrow().value.0;
        Ok(())
    }

    fn value(&self) -> &Self::Type {
        self.value.as_ref()
    }

    fn take_value(self) -> Self::Type {
        self.value.into_inner()
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
    fn divide_assign(&mut self, rhs: &Self, _divide_type: DivideType) -> Fallible<()> {
        // May panic if rhs.value == 0.0
        if rhs.value.0 == 0.0 {
            bail!("Division by 0")
        }
        self.value.0 /= rhs.value.0;
        Ok(())
    }
}

impl_quantize_f32!(ProbabilityWeight);

partial_eq_and_hash_f32!(ProbabilityWeight);
