use crate::algorithms::FactorIterator;
use crate::semirings::Semiring;
use std::marker::PhantomData;

pub struct IdentityFactor<W> {
    ghost: PhantomData<W>,
}

impl<W: Semiring> FactorIterator<W> for IdentityFactor<W> {
    fn new(weight: W) -> Self {
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
