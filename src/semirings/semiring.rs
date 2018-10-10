use std::ops::{Add, AddAssign, Mul, MulAssign};

/// For some operations, the weight set associated to a wFST must have the structure of a semiring.
/// (S, +, *, 0, 1) is a semiring if (S, +, 0) is a commutative monoid with identity element 0,
/// (S, *, 1) is a monoid with identity element 1, * distributes over +, 0 is an annihilator for *.
/// Thus, a semiring is a ring that may lack negation.
pub trait Semiring: Clone + PartialEq + Default + Add + AddAssign + Mul + MulAssign {
    type Type;
    fn plus(&self, rhs: &Self) -> Self;
    fn times(&self, rhs: &Self) -> Self;
    fn zero() -> Self;
    fn one() -> Self;
    fn value(&self) -> Self::Type;
    fn from_value(Self::Type) -> Self;
    fn set_value(&mut self, Self::Type);
}

/// A semiring is said to be divisible if all non-0 elements admit an inverse, that is if S-{0} is a group.
/// (S, +, *, 0, 1) is said to be weakly divisible if for any x and y in S such that x + y != 0,
/// there exists at least one z such that x = (x+y)*z
pub trait WeaklyDivisibleSemiring: Semiring {
    /// Inverse for the * operation
    fn inverse(&self) -> Self;
    // TODO : Not always commutative
    fn divide(&self, rhs: &Self) -> Self;
}

macro_rules! add_mul_semiring {
    ($semiring:ty) => {
        impl Add for $semiring {
            type Output = $semiring;

            fn add(self, other: $semiring) -> $semiring {
                self.plus(&other)
            }
        }

        impl AddAssign for $semiring {
            fn add_assign(&mut self, other: $semiring) {
                let new_value = self.plus(&other).value();
                self.set_value(new_value);
            }
        }

        impl Mul for $semiring {
            type Output = $semiring;

            fn mul(self, other: $semiring) -> $semiring {
                self.times(&other)
            }
        }

        impl MulAssign for $semiring {
            fn mul_assign(&mut self, other: $semiring) {
                let new_value = self.times(&other).value();
                self.set_value(new_value)
            }
        }
    };
}
