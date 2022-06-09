use crate::algorithms::factor_weight::FactorIterator;
use crate::semirings::Semiring;
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Clone)]
/// Trivial factor. Doesn't factor anything.
pub struct IdentityFactor<W> {
    ghost: PhantomData<W>,
}

impl<W> Default for IdentityFactor<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<W> IdentityFactor<W> {
    pub fn new() -> Self {
        Self { ghost: PhantomData }
    }
}

impl<W: Semiring> FactorIterator<W> for IdentityFactor<W> {
    fn new(_weight: W) -> Self {
        Self { ghost: PhantomData }
    }

    fn done(&self) -> bool {
        true
    }
}

impl<W: Semiring> Iterator for IdentityFactor<W> {
    type Item = (W, W);

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
