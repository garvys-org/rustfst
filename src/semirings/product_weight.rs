use crate::semirings::{Semiring, StarSemiring, WeaklyDivisibleSemiring, WeightQuantize};

#[derive(Debug, Eq, PartialOrd, PartialEq, Clone, Default, Hash)]
pub struct ProductWeight<W1, W2>
where
    W1: Semiring,
    W2: Semiring,
{
    weight1: W1,
    weight2: W2,
}

use std::fmt;
use std::fmt::Debug;
impl<W1, W2> fmt::Display for ProductWeight<W1, W2>
where
    W1: Semiring,
    W2: Semiring,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (&self.weight1, &self.weight2).fmt(f)
    }
}

impl<W1, W2> AsRef<Self> for ProductWeight<W1, W2>
where
    W1: Semiring,
    W2: Semiring,
{
    fn as_ref(&self) -> &ProductWeight<W1, W2> {
        &self
    }
}

impl<W1, W2> Semiring for ProductWeight<W1, W2>
where
    W1: Semiring,
    W2: Semiring,
{
    type Type = (W1, W2);

    fn zero() -> Self {
        Self {
            weight1: W1::zero(),
            weight2: W2::zero(),
        }
    }

    fn one() -> Self {
        Self {
            weight1: W1::one(),
            weight2: W2::one(),
        }
    }

    fn new(value: <Self as Semiring>::Type) -> Self {
        Self {
            weight1: value.0,
            weight2: value.1,
        }
    }

    fn plus_assign<P: AsRef<Self>>(&mut self, rhs: P) {
        self.weight1.plus_assign(&rhs.as_ref().weight1);
        self.weight2.plus_assign(&rhs.as_ref().weight2);
    }

    fn times_assign<P: AsRef<Self>>(&mut self, rhs: P) {
        self.weight1.times_assign(&rhs.as_ref().weight1);
        self.weight2.times_assign(&rhs.as_ref().weight2);
    }

    fn value(&self) -> <Self as Semiring>::Type {
        (self.weight1.clone(), self.weight2.clone())
    }

    fn set_value(&mut self, value: <Self as Semiring>::Type) {
        self.weight1 = value.0;
        self.weight2 = value.1;
    }
}
