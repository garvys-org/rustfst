use std::borrow::Borrow;
use std::f32;
use std::hash::{Hash, Hasher};
use std::io::Write;

use anyhow::Result;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::combinator::map;
use nom::number::complete::float;
use nom::IResult;
use ordered_float::OrderedFloat;

use crate::parsers::nom_utils::NomCustomError;
use crate::parsers::parse_bin_f32;
use crate::parsers::write_bin_f32;
use crate::semirings::semiring::SerializableSemiring;
use crate::semirings::utils_float::float_approx_equal;
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

    fn approx_equal<P: Borrow<Self>>(&self, rhs: P, delta: f32) -> bool {
        float_approx_equal(self.value.0, rhs.borrow().value.0, delta)
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
        Ok(*self)
    }
}

impl AsRef<TropicalWeight> for TropicalWeight {
    fn as_ref(&self) -> &TropicalWeight {
        self
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

    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>> {
        let (i, weight) = parse_bin_f32(i)?;
        Ok((i, Self::new(weight)))
    }

    fn write_binary<F: Write>(&self, file: &mut F) -> Result<()> {
        write_bin_f32(file, *self.value())
    }

    fn parse_text(i: &str) -> IResult<&str, Self> {
        // FIXME: nom 7 does not fully parse "infinity", therefore it is done manually
        // even after https://github.com/rust-bakery/nom/pull/1673 wass merged this issue persisted
        // https://github.com/Garvys/rustfst/pull/253#discussion_r1494208294
        let (i, f) = alt((map(tag_no_case("infinity"), |_| f32::INFINITY), float))(i)?;
        Ok((i, Self::new(f)))
    }
}

test_semiring_serializable!(
    tests_tropical_weight_serializable,
    TropicalWeight,
    TropicalWeight::one() TropicalWeight::zero() TropicalWeight::new(0.3) TropicalWeight::new(0.5) TropicalWeight::new(0.0) TropicalWeight::new(-1.2)
);

impl From<f32> for TropicalWeight {
    fn from(f: f32) -> Self {
        Self::new(f)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::hash::BuildHasherDefault;

    use super::*;
    use crate::semirings::{LogWeight, ProbabilityWeight};
    use crate::KDELTA;

    /// Sends every key to one bucket, so a set tells its keys apart by `==` alone.
    #[derive(Default)]
    struct OneBucket;

    impl Hasher for OneBucket {
        fn finish(&self) -> u64 {
            0
        }
        fn write(&mut self, _: &[u8]) {}
    }

    /// Two weights closer than `KDELTA` are two keys however the set hashes them.
    fn close_weights_are_two_keys<W: Semiring<Type = f32>>() {
        let (a, b) = (W::new(1.0), W::new(1.0 + 1.0 / 4096.0));
        assert!(a.approx_equal(&b, KDELTA));
        let mut hashed: HashSet<W> = HashSet::new();
        hashed.insert(a.clone());
        hashed.insert(b.clone());
        let mut one_bucket: HashSet<W, BuildHasherDefault<OneBucket>> = HashSet::default();
        one_bucket.insert(a);
        one_bucket.insert(b);
        assert_eq!(hashed.len(), 2);
        assert_eq!(one_bucket.len(), 2);
    }

    #[test]
    fn test_close_float_weights_are_two_keys() {
        close_weights_are_two_keys::<TropicalWeight>();
        close_weights_are_two_keys::<LogWeight>();
        close_weights_are_two_keys::<ProbabilityWeight>();
    }
}
