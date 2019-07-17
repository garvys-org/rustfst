use failure::Fallible;

use crate::semirings::{CompleteSemiring, Semiring, SemiringProperties, StarSemiring};

/// Boolean semiring: (&, |, false, true).
#[derive(Clone, Debug, PartialEq, PartialOrd, Default, Eq, Copy, Hash)]
pub struct BooleanWeight {
    value: bool,
}

impl Semiring for BooleanWeight {
    type Type = bool;
    type ReverseWeight = BooleanWeight;

    fn zero() -> Self {
        Self { value: false }
    }
    fn one() -> Self {
        Self { value: true }
    }

    fn new(value: <Self as Semiring>::Type) -> Self {
        BooleanWeight { value }
    }

    fn plus_assign<P: AsRef<Self>>(&mut self, rhs: P) -> Fallible<()> {
        self.value |= rhs.as_ref().value;
        Ok(())
    }
    fn times_assign<P: AsRef<Self>>(&mut self, rhs: P) -> Fallible<()> {
        self.value &= rhs.as_ref().value;
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
            | SemiringProperties::IDEMPOTENT
            | SemiringProperties::PATH
    }
}

impl AsRef<BooleanWeight> for BooleanWeight {
    fn as_ref(&self) -> &BooleanWeight {
        &self
    }
}

display_semiring!(BooleanWeight);

impl CompleteSemiring for BooleanWeight {}

impl StarSemiring for BooleanWeight {
    fn closure(&self) -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_weight() -> Fallible<()> {
        let b_true = BooleanWeight::new(true);
        let b_false = BooleanWeight::new(false);

        // Test plus
        assert_eq!(b_true.plus(&b_true)?, b_true);
        assert_eq!(b_true.plus(&b_false)?, b_true);
        assert_eq!(b_false.plus(&b_true)?, b_true);
        assert_eq!(b_false.plus(&b_false)?, b_false);

        // Test times
        assert_eq!(b_true.times(&b_true)?, b_true);
        assert_eq!(b_true.times(&b_false)?, b_false);
        assert_eq!(b_false.times(&b_true)?, b_false);
        assert_eq!(b_false.times(&b_false)?, b_false);
        Ok(())
    }
}
