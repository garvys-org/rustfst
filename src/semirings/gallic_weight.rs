use crate::semirings::ProductWeight;
use crate::semirings::Semiring;
use crate::semirings::{
    StringWeightLeft, StringWeightRestrict, StringWeightRight, UnionWeight, UnionWeightOption,
};

#[derive(PartialOrd, PartialEq, Eq, Clone, Default, Hash, Debug)]
pub struct GallicWeightLeft<W>
where
    W: Semiring,
{
    weight: ProductWeight<StringWeightLeft, W>,
}

#[derive(PartialOrd, PartialEq, Eq, Clone, Default, Hash, Debug)]
pub struct GallicWeightRight<W>
where
    W: Semiring,
{
    weight: ProductWeight<StringWeightRight, W>,
}

#[derive(PartialOrd, PartialEq, Eq, Clone, Default, Hash, Debug)]
pub struct GallicWeightRestrict<W>
where
    W: Semiring,
{
    weight: ProductWeight<StringWeightRestrict, W>,
}

#[derive(PartialOrd, PartialEq, Eq, Clone, Default, Hash, Debug)]
pub struct GallicWeightMin<W>
where
    W: Semiring,
{
    weight: ProductWeight<StringWeightRestrict, W>,
}

fn natural_less<W: Semiring>(w1: &W, w2: &W) -> bool {
    (&w1.plus(w2) == w1) && (w1 != w2)
}

pub enum GallicType {
    GallicLeft,
    GallicRight,
    GallicRestrict,
    GallicMin,
}

macro_rules! gallic_weight {
    ($semiring: ty, $string_weight: ty, $gallic_type: expr) => {
        impl<W> std::fmt::Display for $semiring
        where
            W: Semiring,
        {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.weight.fmt(f)
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

        impl<W> Semiring for $semiring
        where
            W: Semiring,
        {
            type Type = ProductWeight<$string_weight, W>;

            fn zero() -> Self {
                Self {
                    weight: ProductWeight::zero(),
                }
            }

            fn one() -> Self {
                Self {
                    weight: ProductWeight::one(),
                }
            }

            fn new(value: Self::Type) -> Self {
                Self { weight: value }
            }

            fn plus_assign<P: AsRef<Self>>(&mut self, rhs: P) {
                match $gallic_type {
                    GallicType::GallicLeft => self.weight.plus_assign(&rhs.as_ref().weight),
                    GallicType::GallicRight => self.weight.plus_assign(&rhs.as_ref().weight),
                    GallicType::GallicRestrict => self.weight.plus_assign(&rhs.as_ref().weight),
                    GallicType::GallicMin => {
                        if !natural_less(self, rhs.as_ref()) {
                            self.set_value(rhs.as_ref().value());
                        }
                    }
                }
            }

            fn times_assign<P: AsRef<Self>>(&mut self, rhs: P) {
                self.weight.times_assign(&rhs.as_ref().weight);
            }

            fn value(&self) -> Self::Type {
                self.weight.clone()
            }

            fn set_value(&mut self, value: Self::Type) {
                self.weight = value;
            }
        }
    };
}

gallic_weight!(
    GallicWeightLeft<W>,
    StringWeightLeft,
    GallicType::GallicLeft
);

gallic_weight!(
    GallicWeightRight<W>,
    StringWeightRight,
    GallicType::GallicRight
);

gallic_weight!(
    GallicWeightRestrict<W>,
    StringWeightRestrict,
    GallicType::GallicRestrict
);

gallic_weight!(
    GallicWeightMin<W>,
    StringWeightRestrict,
    GallicType::GallicMin
);
use std::marker::PhantomData;
#[derive(Debug, Hash, Default, Clone, PartialEq, PartialOrd, Eq)]
pub struct GallicUnionWeightOption<W> {
    ghost: PhantomData<W>,
}

impl<W: Semiring> UnionWeightOption<GallicWeightRestrict<W>>
    for GallicUnionWeightOption<GallicWeightRestrict<W>>
{
    fn compare(w1: &GallicWeightRestrict<W>, w2: &GallicWeightRestrict<W>) -> bool {
        let s1 = &w1.weight.value1();
        let s2 = &w2.weight.value1();
        let n1 = s1.len();
        let n2 = s2.len();
        if n1 < n2 {
            return true;
        } else if n1 > n2 {
            return false;
        } else {
            // n1 == n2
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
            return false;
        }
    }

    fn merge(
        w1: &GallicWeightRestrict<W>,
        w2: &GallicWeightRestrict<W>,
    ) -> GallicWeightRestrict<W> {
        let p = ProductWeight::new((
            w1.weight.value1().clone(),
            w1.weight.value2().plus(&w2.weight.value2()),
        ));
        GallicWeightRestrict { weight: p }
    }
}

pub type GallicWeight<W> =
    UnionWeight<GallicWeightRestrict<W>, GallicUnionWeightOption<GallicWeightRestrict<W>>>;
