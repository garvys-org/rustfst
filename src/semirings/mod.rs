
pub trait Semiring: Clone + PartialEq + Default {
    fn plus(&self, rhs: &Self) -> Self;
    fn times(&self, rhs: &Self) -> Self;
    fn zero() -> Self;
    fn one() -> Self;
    fn inverse(&self) -> Self;
    fn divide(&self, rhs: &Self) -> Self;
}

// pub mod integer_weight;
pub mod probability_weight;
