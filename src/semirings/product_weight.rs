use std::fmt;
use std::fmt::Debug;

use crate::semirings::{DivideType, Semiring, WeaklyDivisibleSemiring};

#[derive(Debug, Eq, PartialOrd, PartialEq, Clone, Default, Hash)]
pub struct ProductWeight<W1, W2>
where
    W1: Semiring,
    W2: Semiring,
{
    weight1: W1,
    weight2: W2,
}

impl<W1, W2> fmt::Display for ProductWeight<W1, W2>
where
    W1: Semiring,
    W2: Semiring,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (&self.value1(), &self.value2()).fmt(f)
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
        (self.value1().clone(), self.value2().clone())
    }

    fn set_value(&mut self, value: <Self as Semiring>::Type) {
        self.set_value1(value.0);
        self.set_value2(value.1);
    }
}

impl<W1, W2> ProductWeight<W1, W2>
where
    W1: Semiring,
    W2: Semiring,
{
    pub fn value1(&self) -> &W1 {
        &self.weight1
    }

    pub fn value2(&self) -> &W2 {
        &self.weight2
    }

    pub fn set_value1(&mut self, new_weight: W1) {
        self.weight1 = new_weight;
    }

    pub fn set_value2(&mut self, new_weight: W2) {
        self.weight2 = new_weight;
    }
}

impl<W1, W2> From<(W1, W2)> for ProductWeight<W1, W2>
where
    W1: Semiring,
    W2: Semiring,
{
    fn from(t: (W1, W2)) -> Self {
        Self::new(t)
    }
}

impl<W1, W2> WeaklyDivisibleSemiring for ProductWeight<W1, W2>
where
    W1: WeaklyDivisibleSemiring,
    W2: WeaklyDivisibleSemiring,
{
    fn divide(&self, rhs: &Self, divide_type: DivideType) -> Self {
        (
            self.value1().divide(&rhs.value1(), divide_type),
            self.value2().divide(&rhs.value2(), divide_type),
        )
            .into()
    }
}
