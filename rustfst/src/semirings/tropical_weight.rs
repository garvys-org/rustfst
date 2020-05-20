use std::borrow::Borrow;
use std::f32;
use std::hash::{Hash, Hasher};
use std::io::Write;

use anyhow::Result;
use nom::number::complete::{float, le_f32};
use nom::IResult;
use ordered_float::OrderedFloat;

use crate::parsers::bin_fst::utils_serialization::write_bin_f32;
use crate::semirings::semiring::SerializableSemiring;
use crate::semirings::{
    CompleteSemiring, DivideType, ReverseBack, Semiring, SemiringProperties, StarSemiring,
    WeaklyDivisibleSemiring, WeightQuantize,
};

/// Tropical semiring: (min, +, inf, 0).
#[derive(Clone, Debug, PartialOrd, Default, Copy, Eq)]
pub struct TropicalWeight {
    value: OrderedFloat<f32>,
}

impl Semiring for TropicalWeight {
    type Type = f32;
    type ReverseWeight = TropicalWeight;

    fn zero() -> Self {
        Self {
            value: OrderedFloat(f32::INFINITY),
        }
    }

    fn one() -> Self {
        Self {
            value: OrderedFloat(0.0),
        }
    }

    fn new(value: <Self as Semiring>::Type) -> Self {
        TropicalWeight {
            value: OrderedFloat(value),
        }
    }

    fn plus_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Result<()> {
        if rhs.borrow().value < self.value {
            self.value = rhs.borrow().value;
        }
        Ok(())
    }

    fn times_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Result<()> {
        let f1 = self.value();
        let f2 = rhs.borrow().value();
        if f1.eq(&f32::INFINITY) {
        } else if f2.eq(&f32::INFINITY) {
            self.value.0 = *f2;
        } else {
            self.value.0 += f2;
        }
        Ok(())
    }

    fn value(&self) -> &Self::Type {
        &self.value.0
    }

    fn take_value(self) -> Self::Type {
        self.value.0
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
            | SemiringProperties::PATH
            | SemiringProperties::IDEMPOTENT
    }
}

impl ReverseBack<TropicalWeight> for TropicalWeight {
    fn reverse_back(&self) -> Result<TropicalWeight> {
        Ok(self.clone())
    }
}

impl AsRef<TropicalWeight> for TropicalWeight {
    fn as_ref(&self) -> &TropicalWeight {
        &self
    }
}

display_semiring!(TropicalWeight);

impl CompleteSemiring for TropicalWeight {}

impl StarSemiring for TropicalWeight {
    fn closure(&self) -> Self {
        if self.value.is_sign_positive() && self.value.is_finite() {
            Self::new(0.0)
        } else {
            Self::new(f32::NEG_INFINITY)
        }
    }
}

impl WeaklyDivisibleSemiring for TropicalWeight {
    fn divide_assign(&mut self, rhs: &Self, _divide_type: DivideType) -> Result<()> {
        self.value.0 -= rhs.value.0;
        Ok(())
    }
}

impl_quantize_f32!(TropicalWeight);

partial_eq_and_hash_f32!(TropicalWeight);

impl SerializableSemiring for TropicalWeight {
    fn weight_type() -> String {
        "tropical".to_string()
    }

    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, weight) = le_f32(i)?;
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

test_semiring_serializable!(
    tests_tropical_weight_serializable,
    TropicalWeight,
    TropicalWeight::one() TropicalWeight::zero() TropicalWeight::new(0.3) TropicalWeight::new(0.5) TropicalWeight::new(0.0) TropicalWeight::new(-1.2)
);

impl Into<TropicalWeight> for f32 {
    fn into(self) -> TropicalWeight {
        TropicalWeight::new(self)
    }
}
