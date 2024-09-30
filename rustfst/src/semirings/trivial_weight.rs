use anyhow::Result;

use crate::semirings::{CompleteSemiring, ReverseBack, Semiring, SemiringProperties, StarSemiring};
use std::borrow::Borrow;

use super::WeaklyDivisibleSemiring;

/// Trivial semiring: (..., ..., (), ()).
///
/// This is useful for defining unweighted transducers.
#[derive(Clone, Debug, PartialEq, PartialOrd, Default, Eq, Copy, Hash)]
pub struct TrivialWeight;

impl Semiring for TrivialWeight {
    type Type = ();

    type ReverseWeight = TrivialWeight;

    fn zero() -> Self {
        Self
    }

    fn one() -> Self {
        Self
    }

    fn new(_value: Self::Type) -> Self {
        Self
    }

    fn plus_assign<P: Borrow<Self>>(&mut self, _rhs: P) -> anyhow::Result<()> {
        Ok(())
    }

    fn times_assign<P: Borrow<Self>>(&mut self, _rhs: P) -> anyhow::Result<()> {
        Ok(())
    }

    fn approx_equal<P: Borrow<Self>>(&self, _rhs: P, _delta: f32) -> bool {
        true
    }

    fn value(&self) -> &Self::Type {
        &()
    }

    fn take_value(self) -> Self::Type {}

    fn set_value(&mut self, _value: Self::Type) {}

    fn reverse(&self) -> anyhow::Result<Self::ReverseWeight> {
        Ok(*self)
    }

    fn properties() -> SemiringProperties {
        SemiringProperties::LEFT_SEMIRING
            | SemiringProperties::RIGHT_SEMIRING
            | SemiringProperties::COMMUTATIVE
            | SemiringProperties::IDEMPOTENT
            | SemiringProperties::PATH
    }
}

impl ReverseBack<TrivialWeight> for TrivialWeight {
    fn reverse_back(&self) -> Result<TrivialWeight> {
        Ok(*self)
    }
}

impl CompleteSemiring for TrivialWeight {}

impl StarSemiring for TrivialWeight {
    fn closure(&self) -> Self {
        Self
    }
}

impl WeaklyDivisibleSemiring for TrivialWeight {
    fn divide_assign(&mut self, _rhs: &Self, _divide_type: super::DivideType) -> Result<()> {
        bail!("Division by 0")
    }
}

impl From<()> for TrivialWeight {
    fn from(_value: ()) -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trivial_weight() -> Result<()> {
        let trivial = TrivialWeight;

        // Test plus
        assert_eq!(trivial.plus(trivial)?, trivial);

        // Test times
        assert_eq!(trivial.times(trivial)?, trivial);
        Ok(())
    }
}
