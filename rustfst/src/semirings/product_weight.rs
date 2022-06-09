use std::borrow::Borrow;
use std::fmt;
use std::fmt::Debug;
use std::io::Write;

use anyhow::Result;
use nom::IResult;

use crate::parsers::nom_utils::NomCustomError;
use crate::semirings::{
    DivideType, ReverseBack, Semiring, SemiringProperties, SerializableSemiring,
    WeaklyDivisibleSemiring, WeightQuantize,
};
#[cfg(test)]
use crate::semirings::{LogWeight, TropicalWeight};

/// Product semiring: W1 * W2.
#[derive(Debug, Eq, PartialOrd, PartialEq, Clone, Default, Hash)]
pub struct ProductWeight<W1, W2>
where
    W1: Semiring,
    W2: Semiring,
{
    pub(crate) weight: (W1, W2),
}

impl<W1, W2> AsRef<Self> for ProductWeight<W1, W2>
where
    W1: Semiring,
    W2: Semiring,
{
    fn as_ref(&self) -> &ProductWeight<W1, W2> {
        self
    }
}

impl<W1, W2> Semiring for ProductWeight<W1, W2>
where
    W1: Semiring,
    W2: Semiring,
{
    type Type = (W1, W2);
    type ReverseWeight = ProductWeight<W1::ReverseWeight, W2::ReverseWeight>;

    fn zero() -> Self {
        Self {
            weight: (W1::zero(), W2::zero()),
        }
    }

    fn one() -> Self {
        Self {
            weight: (W1::one(), W2::one()),
        }
    }

    fn new(weight: <Self as Semiring>::Type) -> Self {
        Self { weight }
    }

    fn plus_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Result<()> {
        self.weight.0.plus_assign(&rhs.borrow().weight.0)?;
        self.weight.1.plus_assign(&rhs.borrow().weight.1)?;
        Ok(())
    }

    fn times_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Result<()> {
        self.weight.0.times_assign(&rhs.borrow().weight.0)?;
        self.weight.1.times_assign(&rhs.borrow().weight.1)?;
        Ok(())
    }

    fn approx_equal<P: Borrow<Self>>(&self, rhs: P, delta: f32) -> bool {
        self.value1().approx_equal(rhs.borrow().value1(), delta)
            && self.value2().approx_equal(rhs.borrow().value2(), delta)
    }

    fn value(&self) -> &<Self as Semiring>::Type {
        &self.weight
    }

    fn take_value(self) -> <Self as Semiring>::Type {
        self.weight
    }

    fn set_value(&mut self, value: <Self as Semiring>::Type) {
        self.set_value1(value.0);
        self.set_value2(value.1);
    }

    fn reverse(&self) -> Result<Self::ReverseWeight> {
        Ok((self.value1().reverse()?, self.value2().reverse()?).into())
    }

    fn properties() -> SemiringProperties {
        W1::properties()
            & W2::properties()
            & (SemiringProperties::LEFT_SEMIRING
                | SemiringProperties::RIGHT_SEMIRING
                | SemiringProperties::COMMUTATIVE
                | SemiringProperties::IDEMPOTENT)
    }
}

impl<W1: Semiring, W2: Semiring> ReverseBack<ProductWeight<W1, W2>>
    for <ProductWeight<W1, W2> as Semiring>::ReverseWeight
{
    fn reverse_back(&self) -> Result<ProductWeight<W1, W2>> {
        Ok((self.value1().reverse_back()?, self.value2().reverse_back()?).into())
    }
}

impl<W1, W2> ProductWeight<W1, W2>
where
    W1: Semiring,
    W2: Semiring,
{
    pub fn value1(&self) -> &W1 {
        &self.weight.0
    }

    pub fn value2(&self) -> &W2 {
        &self.weight.1
    }

    pub fn set_value1(&mut self, new_weight: W1) {
        self.weight.0 = new_weight;
    }

    pub fn set_value2(&mut self, new_weight: W2) {
        self.weight.1 = new_weight;
    }
}

impl<W1, W2> From<(W1, W2)> for ProductWeight<W1, W2>
where
    W1: Semiring,
    W2: Semiring,
{
    fn from(t: (W1, W2)) -> Self {
        Self::new(t)
    }
}

impl<W1, W2> WeaklyDivisibleSemiring for ProductWeight<W1, W2>
where
    W1: WeaklyDivisibleSemiring,
    W2: WeaklyDivisibleSemiring,
{
    fn divide_assign(&mut self, rhs: &Self, divide_type: DivideType) -> Result<()> {
        self.weight.0.divide_assign(&rhs.weight.0, divide_type)?;
        self.weight.1.divide_assign(&rhs.weight.1, divide_type)?;
        Ok(())
    }
}

impl<W1, W2> WeightQuantize for ProductWeight<W1, W2>
where
    W1: WeightQuantize,
    W2: WeightQuantize,
{
    fn quantize_assign(&mut self, delta: f32) -> Result<()> {
        self.set_value1(self.value1().quantize(delta)?);
        self.set_value2(self.value2().quantize(delta)?);
        Ok(())
    }
}

impl<W1, W2> fmt::Display for ProductWeight<W1, W2>
where
    W1: SerializableSemiring,
    W2: SerializableSemiring,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{}", self.value1(), self.value2())?;
        Ok(())
    }
}

impl<W1, W2> SerializableSemiring for ProductWeight<W1, W2>
where
    W1: SerializableSemiring,
    W2: SerializableSemiring,
{
    fn weight_type() -> String {
        format!("{}_X_{}", W1::weight_type(), W2::weight_type())
    }

    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>> {
        let (i, weight_1) = W1::parse_binary(i)?;
        let (i, weight_2) = W2::parse_binary(i)?;
        Ok((i, Self::new((weight_1, weight_2))))
    }

    fn write_binary<F: Write>(&self, file: &mut F) -> Result<()> {
        self.value1().write_binary(file)?;
        self.value2().write_binary(file)?;
        Ok(())
    }

    fn parse_text(i: &str) -> IResult<&str, Self> {
        let (i, weight_1) = W1::parse_text(i)?;
        let (i, _) = nom::bytes::complete::tag(",")(i)?;
        let (i, weight_2) = W2::parse_text(i)?;
        Ok((i, Self::new((weight_1, weight_2))))
    }
}

test_semiring_serializable!(
    tests_product_weight_serializable,
    ProductWeight::<TropicalWeight, LogWeight>,
    ProductWeight::new((TropicalWeight::new(0.2), LogWeight::new(1.7)))
);
