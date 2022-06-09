use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::marker::PhantomData;

use anyhow::Result;
use nom::IResult;

use crate::parsers::nom_utils::NomCustomError;
use crate::semirings::Semiring;
#[cfg(test)]
use crate::semirings::TropicalWeight;
use crate::semirings::{
    DivideType, SemiringProperties, SerializableSemiring, StringWeightLeft, StringWeightRestrict,
    StringWeightRight, UnionWeight, UnionWeightOption, WeaklyDivisibleSemiring, WeightQuantize,
};
use crate::semirings::{ProductWeight, ReverseBack};
use crate::Label;

/// Product of StringWeightLeft and an arbitrary weight.
#[derive(PartialOrd, PartialEq, Eq, Clone, Hash, Debug)]
pub struct GallicWeightLeft<W>(ProductWeight<StringWeightLeft, W>)
where
    W: Semiring;

/// Product of StringWeightRight and an arbitrary weight.
#[derive(PartialOrd, PartialEq, Eq, Clone, Hash, Debug)]
pub struct GallicWeightRight<W>(ProductWeight<StringWeightRight, W>)
where
    W: Semiring;

/// Product of StringWeighRestrict and an arbitrary weight.
#[derive(PartialOrd, PartialEq, Eq, Clone, Hash, Debug)]
pub struct GallicWeightRestrict<W>(ProductWeight<StringWeightRestrict, W>)
where
    W: Semiring;

/// Product of StringWeightRestrict and an arbitrary weight.
#[derive(PartialOrd, PartialEq, Eq, Clone, Hash, Debug)]
pub struct GallicWeightMin<W>(ProductWeight<StringWeightRestrict, W>)
where
    W: Semiring;

fn natural_less<W: Semiring>(w1: &W, w2: &W) -> Result<bool> {
    Ok((&w1.plus(w2)? == w1) && (w1 != w2))
}

#[allow(clippy::enum_variant_names)]
pub enum GallicType {
    GallicLeft,
    GallicRight,
    GallicRestrict,
    GallicMin,
}

macro_rules! gallic_weight {
    ($semiring: ty, $string_weight: ty, $gallic_type: expr, $reverse_semiring: ty) => {
        impl<W> std::fmt::Display for $semiring
        where
            W: SerializableSemiring,
        {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl<W> AsRef<$semiring> for $semiring
        where
            W: Semiring,
        {
            fn as_ref(&self) -> &Self {
                &self
            }
        }

        impl<W: Semiring> ReverseBack<$semiring> for <$semiring as Semiring>::ReverseWeight {
            fn reverse_back(&self) -> Result<$semiring> {
                Ok(<$semiring>::new(self.0.reverse_back()?))
            }
        }

        impl<W> Semiring for $semiring
        where
            W: Semiring,
        {
            type Type = ProductWeight<$string_weight, W>;
            type ReverseWeight = $reverse_semiring;

            fn zero() -> Self {
                Self(ProductWeight::zero())
            }

            fn one() -> Self {
                Self(ProductWeight::one())
            }

            fn new(value: Self::Type) -> Self {
                Self(value)
            }

            fn plus_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Result<()> {
                match $gallic_type {
                    GallicType::GallicLeft => self.0.plus_assign(&rhs.borrow().0)?,
                    GallicType::GallicRight => self.0.plus_assign(&rhs.borrow().0)?,
                    GallicType::GallicRestrict => self.0.plus_assign(&rhs.borrow().0)?,
                    GallicType::GallicMin => {
                        if !natural_less(self.value2(), rhs.borrow().value2())? {
                            self.set_value(rhs.borrow().value().clone());
                        }
                    }
                };
                Ok(())
            }

            fn times_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Result<()> {
                self.0.times_assign(&rhs.borrow().0)
            }

            fn approx_equal<P: Borrow<Self>>(&self, rhs: P, delta: f32) -> bool {
                self.0.approx_equal(&rhs.borrow().0, delta)
            }

            fn value(&self) -> &Self::Type {
                &self.0
            }

            fn take_value(self) -> Self::Type {
                self.0
            }

            fn set_value(&mut self, value: Self::Type) {
                self.0 = value;
            }

            fn reverse(&self) -> Result<Self::ReverseWeight> {
                Ok(Self::ReverseWeight::new(self.0.reverse()?))
            }

            fn properties() -> SemiringProperties {
                ProductWeight::<$string_weight, W>::properties()
            }
        }

        impl<W> $semiring
        where
            W: Semiring,
        {
            pub fn value1(&self) -> &$string_weight {
                &self.0.value1()
            }

            pub fn value2(&self) -> &W {
                &self.0.value2()
            }

            pub fn set_value1(&mut self, new_weight: $string_weight) {
                self.0.set_value1(new_weight);
            }

            pub fn set_value2(&mut self, new_weight: W) {
                self.0.set_value2(new_weight)
            }
        }

        impl<W> From<($string_weight, W)> for $semiring
        where
            W: Semiring,
        {
            fn from(w: ($string_weight, W)) -> Self {
                Self::new(w.into())
            }
        }

        impl<W> From<(Vec<Label>, W)> for $semiring
        where
            W: Semiring,
        {
            fn from(w: (Vec<Label>, W)) -> Self {
                let (w1, w2) = w;
                Self::new((w1.into(), w2).into())
            }
        }

        impl<W> From<(Label, W)> for $semiring
        where
            W: Semiring,
        {
            fn from(w: (Label, W)) -> Self {
                let (w1, w2) = w;
                Self::new((w1.into(), w2).into())
            }
        }

        impl<W> WeaklyDivisibleSemiring for $semiring
        where
            W: WeaklyDivisibleSemiring,
        {
            fn divide_assign(&mut self, rhs: &Self, divide_type: DivideType) -> Result<()> {
                self.0
                    .weight
                    .0
                    .divide_assign(&rhs.0.weight.0, divide_type)?;
                self.0
                    .weight
                    .1
                    .divide_assign(&rhs.0.weight.1, divide_type)?;
                Ok(())
            }
        }

        impl<W> WeightQuantize for $semiring
        where
            W: WeightQuantize,
        {
            fn quantize_assign(&mut self, delta: f32) -> Result<()> {
                self.0.quantize_assign(delta)
            }
        }

        impl<W: SerializableSemiring> SerializableSemiring for $semiring {
            fn weight_type() -> String {
                match $gallic_type {
                    GallicType::GallicLeft => "left_gallic".to_string(),
                    GallicType::GallicRight => "right_gallic".to_string(),
                    GallicType::GallicRestrict => "restricted_gallic".to_string(),
                    GallicType::GallicMin => "min_gallic".to_string(),
                }
            }

            fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>> {
                let (i, w) = ProductWeight::<$string_weight, W>::parse_binary(i)?;
                Ok((i, Self(w)))
            }

            fn write_binary<F: Write>(&self, file: &mut F) -> Result<()> {
                self.0.write_binary(file)
            }

            fn parse_text(i: &str) -> IResult<&str, Self> {
                let (i, w) = ProductWeight::<$string_weight, W>::parse_text(i)?;
                Ok((i, Self(w)))
            }
        }
    };
}

gallic_weight!(
    GallicWeightLeft<W>,
    StringWeightLeft,
    GallicType::GallicLeft,
    GallicWeightRight<W::ReverseWeight>
);

gallic_weight!(
    GallicWeightRight<W>,
    StringWeightRight,
    GallicType::GallicRight,
    GallicWeightLeft<W::ReverseWeight>
);

gallic_weight!(
    GallicWeightRestrict<W>,
    StringWeightRestrict,
    GallicType::GallicRestrict,
    GallicWeightRestrict<W::ReverseWeight>
);

gallic_weight!(
    GallicWeightMin<W>,
    StringWeightRestrict,
    GallicType::GallicMin,
    GallicWeightMin<W::ReverseWeight>
);
#[derive(Debug, Hash, Clone, PartialEq, PartialOrd, Eq)]
pub struct GallicUnionWeightOption<W> {
    ghost: PhantomData<W>,
}

impl<W: Semiring> UnionWeightOption<GallicWeightRestrict<W>>
    for GallicUnionWeightOption<GallicWeightRestrict<W>>
{
    type ReverseOptions = GallicUnionWeightOption<GallicWeightRestrict<W::ReverseWeight>>;

    fn compare(w1: &GallicWeightRestrict<W>, w2: &GallicWeightRestrict<W>) -> bool {
        let s1 = w1.0.value1();
        let s2 = w2.0.value1();
        let n1 = s1.len_labels();
        let n2 = s2.len_labels();

        match n1.cmp(&n2) {
            Ordering::Less => true,
            Ordering::Greater => false,
            Ordering::Equal => {
                if n1 == 0 {
                    return false;
                }
                let v1 = s1.value.unwrap_labels();
                let v2 = s2.value.unwrap_labels();
                for i in 0..n1 {
                    let l1 = v1[i];
                    let l2 = v2[i];
                    if l1 < l2 {
                        return true;
                    }
                    if l1 > l2 {
                        return false;
                    }
                }
                false
            }
        }
    }

    fn merge(
        w1: &GallicWeightRestrict<W>,
        w2: &GallicWeightRestrict<W>,
    ) -> Result<GallicWeightRestrict<W>> {
        let p = ProductWeight::new((w1.0.value1().clone(), w1.0.value2().plus(w2.0.value2())?));
        Ok(GallicWeightRestrict(p))
    }
}

/// UnionWeight of GallicWeightRestrict.
#[derive(Debug, PartialOrd, PartialEq, Clone, Hash, Eq)]
pub struct GallicWeight<W>(
    pub UnionWeight<GallicWeightRestrict<W>, GallicUnionWeightOption<GallicWeightRestrict<W>>>,
)
where
    W: Semiring;

impl<W> Display for GallicWeight<W>
where
    W: SerializableSemiring,
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<W> AsRef<GallicWeight<W>> for GallicWeight<W>
where
    W: Semiring,
{
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<W: Semiring> Semiring for GallicWeight<W> {
    type Type = Vec<GallicWeightRestrict<W>>;
    type ReverseWeight = GallicWeight<W::ReverseWeight>;

    fn zero() -> Self {
        Self(UnionWeight::zero())
    }

    fn one() -> Self {
        Self(UnionWeight::one())
    }

    fn new(value: Self::Type) -> Self {
        Self(UnionWeight::new(value))
    }

    fn plus_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Result<()> {
        self.0.plus_assign(&rhs.borrow().0)
    }

    fn times_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Result<()> {
        self.0.times_assign(&rhs.borrow().0)
    }

    fn approx_equal<P: Borrow<Self>>(&self, rhs: P, delta: f32) -> bool {
        self.0.approx_equal(&rhs.borrow().0, delta)
    }

    fn value(&self) -> &Self::Type {
        self.0.value()
    }

    fn take_value(self) -> Self::Type {
        self.0.take_value()
    }

    fn set_value(&mut self, value: Self::Type) {
        self.0.set_value(value)
    }

    fn reverse(&self) -> Result<Self::ReverseWeight> {
        Ok(GallicWeight(self.0.reverse()?))
    }

    fn properties() -> SemiringProperties {
        UnionWeight::<GallicWeightRestrict<W>, GallicUnionWeightOption<GallicWeightRestrict<W>>>::properties()
    }
}

impl<W: Semiring> ReverseBack<GallicWeight<W>> for <GallicWeight<W> as Semiring>::ReverseWeight {
    fn reverse_back(&self) -> Result<GallicWeight<W>> {
        Ok(GallicWeight(self.0.reverse_back()?))
    }
}

impl<W: Semiring> GallicWeight<W> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &GallicWeightRestrict<W>> {
        self.0.iter()
    }
}

impl<W> From<(StringWeightRestrict, W)> for GallicWeight<W>
where
    W: Semiring,
{
    fn from(w: (StringWeightRestrict, W)) -> Self {
        let (w1, w2) = w;
        let mut gw = GallicWeightRestrict::one();
        gw.set_value1(w1);
        gw.set_value2(w2);
        Self::new(vec![gw])
    }
}

impl<W> From<GallicWeightRestrict<W>> for GallicWeight<W>
where
    W: Semiring,
{
    fn from(w: GallicWeightRestrict<W>) -> Self {
        Self::new(vec![w])
    }
}

impl<W> From<(Vec<Label>, W)> for GallicWeight<W>
where
    W: Semiring,
{
    fn from(w: (Vec<Label>, W)) -> Self {
        let (w1, w2) = w;
        let a: StringWeightRestrict = w1.into();
        (a, w2).into()
    }
}

impl<W> From<(Label, W)> for GallicWeight<W>
where
    W: Semiring,
{
    fn from(w: (Label, W)) -> Self {
        let (w1, w2) = w;
        (vec![w1], w2).into()
    }
}

impl<W> WeaklyDivisibleSemiring for GallicWeight<W>
where
    W: WeaklyDivisibleSemiring,
{
    fn divide_assign(&mut self, rhs: &Self, divide_type: DivideType) -> Result<()> {
        self.0.divide_assign(&rhs.0, divide_type)?;
        Ok(())
    }
}

impl<W> WeightQuantize for GallicWeight<W>
where
    W: WeightQuantize,
{
    fn quantize_assign(&mut self, delta: f32) -> Result<()> {
        self.0.quantize_assign(delta)
    }
}

impl<W: SerializableSemiring> SerializableSemiring for GallicWeight<W> {
    fn weight_type() -> String {
        "gallic".to_string()
    }

    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>> {
        let (i, w) = UnionWeight::<
            GallicWeightRestrict<W>,
            GallicUnionWeightOption<GallicWeightRestrict<W>>,
        >::parse_binary(i)?;
        Ok((i, Self(w)))
    }

    fn write_binary<F: Write>(&self, file: &mut F) -> Result<()> {
        self.0.write_binary(file)
    }

    fn parse_text(i: &str) -> IResult<&str, Self> {
        let (i, w) = UnionWeight::<
            GallicWeightRestrict<W>,
            GallicUnionWeightOption<GallicWeightRestrict<W>>,
        >::parse_text(i)?;
        Ok((i, Self(w)))
    }
}

test_semiring_serializable!(
    test_gallic_weight_left_serializable,
    GallicWeightLeft<TropicalWeight>,
    GallicWeightLeft::one()
    GallicWeightLeft::zero()
    GallicWeightLeft::from((vec![1,2],TropicalWeight::new(0.3)))
);

test_semiring_serializable!(
    test_gallic_weight_right_serializable,
    GallicWeightRight<TropicalWeight>,
    GallicWeightRight::one()
    GallicWeightRight::zero()
    GallicWeightRight::from((vec![1,2],TropicalWeight::new(0.3)))
);

test_semiring_serializable!(
    test_gallic_weight_restrict_serializable,
    GallicWeightRestrict<TropicalWeight>,
    GallicWeightRestrict::one()
    GallicWeightRestrict::zero()
    GallicWeightRestrict::from((vec![1,2],TropicalWeight::new(0.3)))
);

test_semiring_serializable!(
    test_gallic_weight_min_serializable,
    GallicWeightMin<TropicalWeight>,
    GallicWeightMin::one()
    GallicWeightMin::zero()
    GallicWeightMin::from((vec![1,2],TropicalWeight::new(0.3)))
);

test_semiring_serializable!(
    test_gallic_weight_serializable,
    GallicWeight<TropicalWeight>,
    GallicWeight::one()
    GallicWeight::zero()
    GallicWeight::from((vec![1,2],TropicalWeight::new(0.3)))
);
