
pub trait Semiring: Clone + PartialEq {
    fn plus(&self, rhs: &Self) -> Self;
    fn times(&self, rhs: &Self) -> Self;
    fn zero() -> Self;
    fn one() -> Self;
}

pub mod probability_weight;
pub mod integer_weight;
