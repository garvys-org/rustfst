use crate::semirings::Semiring;

/// A factor iterator takes as argument a weight w and returns a sequence of
/// pairs of weights (xi, yi) such that the sum of the products xi times yi is
/// equal to w. If w is fully factored, the iterator should return nothing.
pub trait FactorIterator<W: Semiring>:
    std::fmt::Debug + PartialEq + Clone + Iterator<Item = (W, W)> + Sync
{
    fn new(weight: W) -> Self;
    fn done(&self) -> bool;
}
