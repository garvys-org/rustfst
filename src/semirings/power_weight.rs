use std::cmp::Ordering;
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;
use std::hash::Hasher;

use generic_array::ArrayLength;
use generic_array::GenericArray;

use crate::semirings::{Semiring, StarSemiring, WeaklyDivisibleSemiring, WeightQuantize};

pub struct PowerWeight<W, N>
where
    W: Semiring,
    N: ArrayLength<W>,
{
    weights: GenericArray<W, N>,
}

impl<W, N> fmt::Display for PowerWeight<W, N>
where
    W: Semiring,
    N: ArrayLength<W>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.weights.as_slice().fmt(f)
    }
}

impl<W, N> fmt::Debug for PowerWeight<W, N>
where
    W: Semiring,
    N: ArrayLength<W>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.weights.as_slice().fmt(f)
    }
}

impl<W, N> Hash for PowerWeight<W, N>
where
    W: Semiring,
    N: ArrayLength<W>,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.weights.as_slice().hash(state);
    }
}

impl<W, N> Default for PowerWeight<W, N>
where
    W: Semiring,
    N: ArrayLength<W>,
{
    fn default() -> Self {
        Self {
            weights: GenericArray::clone_from_slice(vec![W::default(); N::to_usize()].as_slice()),
        }
    }
}

impl<W, N> Clone for PowerWeight<W, N>
where
    W: Semiring,
    N: ArrayLength<W>,
{
    fn clone(&self) -> Self {
        PowerWeight {
            weights: self.weights.clone(),
        }
    }
}

impl<W, N> PartialOrd for PowerWeight<W, N>
where
    W: Semiring,
    N: ArrayLength<W>,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.weights.partial_cmp(&other.weights)
    }
}

impl<W, N> PartialEq for PowerWeight<W, N>
where
    W: Semiring,
    N: ArrayLength<W>,
{
    fn eq(&self, other: &Self) -> bool {
        self.weights.eq(&other.weights)
    }
}

impl<W, N> AsRef<Self> for PowerWeight<W, N>
where
    W: Semiring,
    N: ArrayLength<W>,
{
    fn as_ref(&self) -> &PowerWeight<W, N> {
        &self
    }
}

impl<W, N> Eq for PowerWeight<W, N>
where
    W: Semiring,
    N: ArrayLength<W>,
{
}

impl<W, N> Semiring for PowerWeight<W, N>
where
    W: Semiring,
    N: ArrayLength<W>,
{
    type Type = GenericArray<W, N>;

    fn zero() -> Self {
        Self {
            weights: GenericArray::clone_from_slice(&[W::zero()]),
        }
    }

    fn one() -> Self {
        Self {
            weights: GenericArray::clone_from_slice(&[W::one()]),
        }
    }

    fn new(value: <Self as Semiring>::Type) -> Self {
        Self { weights: value }
    }

    fn plus_assign<P: AsRef<Self>>(&mut self, rhs: P) {
        for i in 0..self.weights.len() {
            self.weights[i].plus_assign(&rhs.as_ref().weights[i]);
        }
    }

    fn times_assign<P: AsRef<Self>>(&mut self, rhs: P) {
        for i in 0..self.weights.len() {
            self.weights[i].times_assign(&rhs.as_ref().weights[i]);
        }
    }

    fn value(&self) -> <Self as Semiring>::Type {
        self.weights.clone()
    }

    fn set_value(&mut self, value: <Self as Semiring>::Type) {
        self.weights = value;
    }
}
