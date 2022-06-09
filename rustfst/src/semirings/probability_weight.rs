use std::borrow::Borrow;
use std::f32;
use std::hash::{Hash, Hasher};
use std::io::Write;

use anyhow::Result;
use nom::number::complete::float;
use nom::IResult;
use ordered_float::OrderedFloat;

use crate::parsers::nom_utils::NomCustomError;
use crate::parsers::parse_bin_f32;
use crate::parsers::write_bin_f32;
use crate::semirings::utils_float::float_approx_equal;
use crate::semirings::{
    CompleteSemiring, DivideType, ReverseBack, Semiring, SemiringProperties, SerializableSemiring,
    StarSemiring, WeaklyDivisibleSemiring, WeightQuantize,
};
use crate::KDELTA;

/// Probability semiring: (x, +, 0.0, 1.0).
#[derive(Clone, Debug, PartialOrd, Default, Copy, Eq)]
pub struct ProbabilityWeight {
    value: OrderedFloat<f32>,
}

impl Semiring for ProbabilityWeight {
    type Type = f32;
    type ReverseWeight = ProbabilityWeight;

    fn zero() -> Self {
        Self {
            value: OrderedFloat(0.0),
        }
    }
    fn one() -> Self {
        Self {
            value: OrderedFloat(1.0),
        }
    }

    fn new(value: <Self as Semiring>::Type) -> Self {
        ProbabilityWeight {
            value: OrderedFloat(value),
        }
    }

    fn plus_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Result<()> {
        self.value.0 += rhs.borrow().value.0;
        Ok(())
    }

    fn times_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Result<()> {
        self.value.0 *= rhs.borrow().value.0;
        Ok(())
    }

    fn approx_equal<P: Borrow<Self>>(&self, rhs: P, delta: f32) -> bool {
        float_approx_equal(self.value.0, rhs.borrow().value.0, delta)
    }

    fn value(&self) -> &Self::Type {
        self.value.as_ref()
    }

    fn take_value(self) -> Self::Type {
        self.value.into_inner()
    }

    fn set_value(&mut self, value: <Self as Semiring>::Type) {
        self.value.0 = value
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

impl ReverseBack<ProbabilityWeight> for ProbabilityWeight {
    fn reverse_back(&self) -> Result<ProbabilityWeight> {
        unimplemented!()
    }
}

impl AsRef<ProbabilityWeight> for ProbabilityWeight {
    fn as_ref(&self) -> &ProbabilityWeight {
        self
    }
}

display_semiring!(ProbabilityWeight);

impl CompleteSemiring for ProbabilityWeight {}

impl SerializableSemiring for ProbabilityWeight {
    fn weight_type() -> String {
        "probability".to_string()
    }

    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>> {
        let (i, weight) = parse_bin_f32(i)?;
        Ok((i, Self::new(weight)))
    }

    fn write_binary<F: Write>(&self, file: &mut F) -> Result<()> {
        write_bin_f32(file, *self.value())
    }

    fn parse_text(i: &str) -> IResult<&str, Self> {
        let (i, f) = float(i)?;
        Ok((i, Self::new(f)))
    }
}

impl StarSemiring for ProbabilityWeight {
    fn closure(&self) -> Self {
        Self::new(1.0 / (1.0 - self.value.0))
    }
}

impl WeaklyDivisibleSemiring for ProbabilityWeight {
    fn divide_assign(&mut self, rhs: &Self, _divide_type: DivideType) -> Result<()> {
        // May panic if rhs.value == 0.0
        if rhs.value.0 == 0.0 {
            bail!("Division by 0")
        }
        self.value.0 /= rhs.value.0;
        Ok(())
    }
}

impl_quantize_f32!(ProbabilityWeight);

partial_eq_and_hash_f32!(ProbabilityWeight);

test_semiring_serializable!(
    tests_probability_weight_serializable,
    ProbabilityWeight,
    ProbabilityWeight::one() ProbabilityWeight::zero() ProbabilityWeight::new(0.3) ProbabilityWeight::new(0.5) ProbabilityWeight::new(0.0) ProbabilityWeight::new(1.0)
);

impl From<f32> for ProbabilityWeight {
    fn from(f: f32) -> Self {
        Self::new(f)
    }
}
