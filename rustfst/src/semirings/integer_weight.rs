use nom::character::complete::i32;
use std::{borrow::Borrow, io::Write};

use anyhow::Result;
use nom::IResult;

use crate::{
    parsers::{parse_bin_i32, write_bin_i32},
    semirings::{CompleteSemiring, ReverseBack, Semiring, SemiringProperties, StarSemiring},
    NomCustomError,
};

use super::SerializableSemiring;

/// Probability semiring: (x, +, 0.0, 1.0).
#[derive(Clone, Debug, PartialEq, PartialOrd, Default, Hash, Eq, Copy)]
pub struct IntegerWeight {
    value: i32,
}

impl Semiring for IntegerWeight {
    type Type = i32;
    type ReverseWeight = IntegerWeight;

    fn zero() -> Self {
        Self { value: 0 }
    }
    fn one() -> Self {
        Self { value: 1 }
    }

    fn new(value: <Self as Semiring>::Type) -> Self {
        IntegerWeight { value }
    }

    fn plus_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Result<()> {
        self.value += rhs.borrow().value;
        Ok(())
    }

    fn times_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Result<()> {
        self.value *= rhs.borrow().value;
        Ok(())
    }

    fn approx_equal<P: Borrow<Self>>(&self, rhs: P, _delta: f32) -> bool {
        self.value == rhs.borrow().value
    }

    fn value(&self) -> &Self::Type {
        &self.value
    }

    fn take_value(self) -> Self::Type {
        self.value
    }

    fn set_value(&mut self, value: <Self as Semiring>::Type) {
        self.value = value
    }

    fn reverse(&self) -> Result<Self::ReverseWeight> {
        Ok(*self)
    }

    fn properties() -> SemiringProperties {
        SemiringProperties::LEFT_SEMIRING
            | SemiringProperties::RIGHT_SEMIRING
            | SemiringProperties::COMMUTATIVE
    }
}

impl ReverseBack<IntegerWeight> for IntegerWeight {
    fn reverse_back(&self) -> Result<IntegerWeight> {
        Ok(*self)
    }
}

impl AsRef<IntegerWeight> for IntegerWeight {
    fn as_ref(&self) -> &IntegerWeight {
        self
    }
}

display_semiring!(IntegerWeight);

impl CompleteSemiring for IntegerWeight {}

impl StarSemiring for IntegerWeight {
    fn closure(&self) -> Self {
        if self.value == 0 {
            return Self::new(1);
        }
        Self::new(i32::max_value())
    }
}

impl From<i32> for IntegerWeight {
    fn from(i: i32) -> Self {
        Self::new(i)
    }
}

impl SerializableSemiring for IntegerWeight {
    fn weight_type() -> String {
        "integer".to_string()
    }

    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>> {
        let (i, weight) = parse_bin_i32(i)?;
        Ok((i, Self::new(weight)))
    }

    fn write_binary<F: Write>(&self, file: &mut F) -> Result<()> {
        write_bin_i32(file, *self.value())
    }

    fn parse_text(i: &str) -> IResult<&str, Self> {
        let (i, f) = i32(i)?;
        Ok((i, Self::new(f)))
    }
}

test_semiring_serializable!(
    tests_integer_weight_serializable,
    IntegerWeight,
    IntegerWeight::one() IntegerWeight::zero() IntegerWeight::new(3) IntegerWeight::new(5) IntegerWeight::new(10) IntegerWeight::new(100)
);
