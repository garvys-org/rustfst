use std::borrow::Borrow;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;

use bitflags::bitflags;

use crate::parsers::nom_utils::NomCustomError;
use anyhow::Result;
use nom::IResult;
use std::io::Write;

bitflags! {
    /// Properties verified by the Semiring.
    pub struct SemiringProperties: u32 {
        /// For all a, b, c: Times(c, Plus(a, b)) = Plus(Times(c, a), Times(c, b)).
        const LEFT_SEMIRING =  0b00001;
        /// For all a, b, c: Times(Plus(a, b), c) = Plus(Times(a, c), Times(b, c)).
        const RIGHT_SEMIRING = 0b00010;
        /// For all a, b: Times(a, b) = Times(b, a).
        const COMMUTATIVE =    0b00100;
        /// For all a: Plus(a, a) = a.
        const IDEMPOTENT =     0b01000;
        /// For all a, b: Plus(a, b) = a or Plus(a, b) = b.
        const PATH =           0b10000;
        const SEMIRING = Self::LEFT_SEMIRING.bits() | Self::RIGHT_SEMIRING.bits();
    }
}

/// The weight on an Fst must implement the `Semiring` trait.
/// Indeed, the weight set associated to a Fst must have the structure of a semiring.
/// `(S, +, *, 0, 1)` is a semiring if `(S, +, 0)` is a commutative monoid with identity element 0,
/// `(S, *, 1)` is a monoid with identity element `1`, `*` distributes over `+`,
/// `0` is an annihilator for `*`.
/// Thus, a semiring is a ring that may lack negation.
/// For more information : <https://cs.nyu.edu/~mohri/pub/hwa.pdf>
pub trait Semiring: Clone + PartialEq + PartialOrd + Debug + Hash + Eq + Sync + 'static {
    type Type: Clone + Debug;
    type ReverseWeight: Semiring + ReverseBack<Self>;

    fn zero() -> Self;
    fn one() -> Self;

    fn new(value: Self::Type) -> Self;

    fn plus<P: Borrow<Self>>(&self, rhs: P) -> Result<Self> {
        let mut w = self.clone();
        w.plus_assign(rhs)?;
        Ok(w)
    }
    fn plus_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Result<()>;

    fn times<P: Borrow<Self>>(&self, rhs: P) -> Result<Self> {
        let mut w = self.clone();
        w.times_assign(rhs)?;
        Ok(w)
    }
    fn times_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Result<()>;

    fn approx_equal<P: Borrow<Self>>(&self, rhs: P, delta: f32) -> bool;

    /// Borrow underneath value.
    fn value(&self) -> &Self::Type;
    /// Move underneath value.
    fn take_value(self) -> Self::Type;
    fn set_value(&mut self, value: Self::Type);
    fn is_one(&self) -> bool {
        *self == Self::one()
    }
    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
    fn reverse(&self) -> Result<Self::ReverseWeight>;
    fn properties() -> SemiringProperties;
}

pub trait ReverseBack<W> {
    fn reverse_back(&self) -> Result<W>;
}

/// Determines direction of division.
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum DivideType {
    /// Left division.
    DivideLeft,
    /// Right division.
    DivideRight,
    /// Division in a commutative semiring.
    DivideAny,
}

/// A semiring is said to be divisible if all non-0 elements admit an inverse,
/// that is if `S-{0}` is a group.
/// `(S, +, *, 0, 1)` is said to be weakly divisible if
/// for any `x` and `y` in `S` such that `x + y != 0`,
/// there exists at least one `z` such that `x = (x+y)*z`.
/// For more information : `https://cs.nyu.edu/~mohri/pub/hwa.pdf`
pub trait WeaklyDivisibleSemiring: Semiring {
    fn divide_assign(&mut self, rhs: &Self, divide_type: DivideType) -> Result<()>;
    fn divide(&self, rhs: &Self, divide_type: DivideType) -> Result<Self> {
        let mut w = self.clone();
        w.divide_assign(rhs, divide_type)?;
        Ok(w)
    }
}

/// A semiring `(S, ⊕, ⊗, 0, 1)` is said to be complete if for any index set `I` and any family
/// `(ai)i ∈ I` of elements of `S`, `⊕(ai)i∈I` is an element of `S` whose definition
/// does not depend on the order of the terms in the ⊕-sum.
/// Note that in a complete semiring all weighted transducers are regulated since all
/// infinite sums are elements of S.
/// For more information : `https://cs.nyu.edu/~mohri/pub/hwa.pdf`
pub trait CompleteSemiring: Semiring {}

/// A complete semiring S is a starsemiring that is a semiring that can be augmented with an
/// internal unary closure operation ∗ defined by `a∗=⊕an (infinite sum) for any a ∈ S`.
/// Furthermore, associativity, commutativity, and distributivity apply to these infinite sums.
/// For more information : `https://cs.nyu.edu/~mohri/pub/hwa.pdf`
pub trait StarSemiring: Semiring {
    fn closure(&self) -> Self;
}

pub trait WeightQuantize: Semiring {
    fn quantize_assign(&mut self, delta: f32) -> Result<()>;
    fn quantize(&self, delta: f32) -> Result<Self> {
        let mut w = self.clone();
        w.quantize_assign(delta)?;
        Ok(w)
    }
}

macro_rules! impl_quantize_f32 {
    ($semiring: ident) => {
        impl WeightQuantize for $semiring {
            fn quantize_assign(&mut self, delta: f32) -> Result<()> {
                let v = *self.value();
                if v.is_infinite() {
                    return Ok(());
                }
                self.set_value(((v / delta) + 0.5).floor() * delta);
                Ok(())
            }
        }
    };
}

macro_rules! display_semiring {
    ($semiring:tt) => {
        use std::fmt;
        impl fmt::Display for $semiring {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.value())?;
                Ok(())
            }
        }
    };
}

macro_rules! partial_eq_and_hash_f32 {
    ($semiring:tt) => {
        impl PartialEq for $semiring {
            fn eq(&self, other: &Self) -> bool {
                // self.value() == other.value()
                let w1 = *self.value();
                let w2 = *other.value();
                w1 <= (w2 + KDELTA) && w2 <= (w1 + KDELTA)
            }
        }

        impl Hash for $semiring {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.value.hash(state)
            }
        }
    };
}

pub trait SerializableSemiring: Semiring + Display {
    fn weight_type() -> String;
    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>>;
    fn write_binary<F: Write>(&self, file: &mut F) -> Result<()>;

    fn parse_text(i: &str) -> IResult<&str, Self>;
    fn write_text<F: Write>(&self, file: &mut F) -> Result<()> {
        // Use implementation of Display trait.
        write!(file, "{}", self)?;
        Ok(())
    }
}
