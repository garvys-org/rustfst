/// For some operations, the weight set associated to a wFST must have the structure of a semiring.
/// (S, +, *, 0, 1) is a semiring if (S, +, 0) is a commutative monoid with identity element 0,
/// (S, *, 1) is a monoid with identity element 1, * distributes over +, 0 is an annihilator for *.
/// Thus, a semiring is a ring that may lack negation.
pub trait Semiring: Clone + PartialEq + Default {
    fn plus(&self, rhs: &Self) -> Self;
    fn times(&self, rhs: &Self) -> Self;
    fn zero() -> Self;
    fn one() -> Self;
}

/// A semiring is said to be divisible if all non-0 elements admit an inverse, that is if S-{0} is a group.
/// (S, +, *, 0, 1) is said to be weakly divisible if for any x and y in S such that x + y != 0,
/// there exists at least one z such that x = (x+y)*z 
pub trait WeaklyDivisibleSemiring : Semiring {
    /// Inverse for the * operation
    fn inverse(&self) -> Self;
    // TODO : Not always commutative
    fn divide(&self, rhs: &Self) -> Self;
}
