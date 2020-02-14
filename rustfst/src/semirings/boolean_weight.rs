use failure::Fallible;

use crate::semirings::{
    CompleteSemiring, IntoSemiring, Semiring, SemiringProperties, StarSemiring,
};
use std::borrow::Borrow;
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

    fn plus_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Fallible<()> {
        self.value |= rhs.borrow().value;
        Ok(())
    }
    fn times_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Fallible<()> {
        self.value &= rhs.borrow().value;
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

impl IntoSemiring<BooleanWeight> for BooleanWeight {
    fn reverse_back(&self) -> Fallible<BooleanWeight> {
        unimplemented!()
    }
}

display_semiring!(BooleanWeight);

impl CompleteSemiring for BooleanWeight {}

impl StarSemiring for BooleanWeight {
    fn closure(&self) -> Self {
        Self::new(true)
    }
}

impl Into<BooleanWeight> for bool {
    fn into(self) -> BooleanWeight {
        BooleanWeight::new(self)
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
