pub trait Semiring: Clone {
    fn plus(&self, rhs: &Self) -> Self;
    fn times(&self, rhs: &Self) -> Self;
    fn zero() -> Self;
    fn one() -> Self;
}

pub mod probability_semiring;
pub mod integer_semiring;
