use crate::semirings::ProductWeight;
use crate::semirings::Semiring;
use crate::semirings::{
    DivideType, StringWeightLeft, StringWeightRestrict, StringWeightRight, UnionWeight,
    UnionWeightOption, WeaklyDivisibleSemiring,
};
use crate::Label;

#[derive(PartialOrd, PartialEq, Eq, Clone, Default, Hash, Debug)]
pub struct GallicWeightLeft<W>(ProductWeight<StringWeightLeft, W>)
where
    W: Semiring;

#[derive(PartialOrd, PartialEq, Eq, Clone, Default, Hash, Debug)]
pub struct GallicWeightRight<W>(ProductWeight<StringWeightRight, W>)
where
    W: Semiring;

#[derive(PartialOrd, PartialEq, Eq, Clone, Default, Hash, Debug)]
pub struct GallicWeightRestrict<W>(ProductWeight<StringWeightRestrict, W>)
where
    W: Semiring;

#[derive(PartialOrd, PartialEq, Eq, Clone, Default, Hash, Debug)]
pub struct GallicWeightMin<W>(ProductWeight<StringWeightRestrict, W>)
where
    W: Semiring;
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

        impl<W> Semiring for $semiring
        where
            W: Semiring,
        {
            type Type = ProductWeight<$string_weight, W>;

            fn zero() -> Self {
                Self(ProductWeight::zero())
            }

            fn one() -> Self {
                Self(ProductWeight::one())
            }

            fn new(value: Self::Type) -> Self {
                Self(value)
            }

            fn plus_assign<P: AsRef<Self>>(&mut self, rhs: P) {
                match $gallic_type {
                    GallicType::GallicLeft => self.0.plus_assign(&rhs.as_ref().0),
                    GallicType::GallicRight => self.0.plus_assign(&rhs.as_ref().0),
                    GallicType::GallicRestrict => self.0.plus_assign(&rhs.as_ref().0),
                    GallicType::GallicMin => {
                        if !natural_less(self, rhs.as_ref()) {
                            self.set_value(rhs.as_ref().value());
                        }
                    }
                }
            }

            fn times_assign<P: AsRef<Self>>(&mut self, rhs: P) {
                self.0.times_assign(&rhs.as_ref().0);
            }

            fn value(&self) -> Self::Type {
                self.0.clone()
            }

            fn set_value(&mut self, value: Self::Type) {
                self.0 = value;
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
            fn divide(&self, rhs: &Self, divide_type: DivideType) -> Self {
                (
                    self.value1().divide(rhs.value1(), divide_type),
                    self.value2().divide(rhs.value2(), divide_type),
                )
                    .into()
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
        let s1 = &w1.0.value1();
        let s2 = &w2.0.value1();
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
        let p = ProductWeight::new((w1.0.value1().clone(), w1.0.value2().plus(&w2.0.value2())));
        GallicWeightRestrict(p)
    }
}

#[derive(Debug, PartialOrd, PartialEq, Clone, Hash, Default, Eq)]
pub struct GallicWeight<W>(
    UnionWeight<GallicWeightRestrict<W>, GallicUnionWeightOption<GallicWeightRestrict<W>>>,
)
where
    W: Semiring;

impl<W> std::fmt::Display for GallicWeight<W>
where
    W: Semiring,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<W> AsRef<GallicWeight<W>> for GallicWeight<W>
where
    W: Semiring,
{
    fn as_ref(&self) -> &Self {
        &self
    }
}

impl<W: Semiring> Semiring for GallicWeight<W> {
    type Type = Vec<GallicWeightRestrict<W>>;

    fn zero() -> Self {
        Self(UnionWeight::zero())
    }

    fn one() -> Self {
        Self(UnionWeight::one())
    }

    fn new(value: Self::Type) -> Self {
        Self(UnionWeight::new(value))
    }

    fn plus_assign<P: AsRef<Self>>(&mut self, rhs: P) {
        self.0.plus_assign(&rhs.as_ref().0)
    }

    fn times_assign<P: AsRef<Self>>(&mut self, rhs: P) {
        self.0.times_assign(&rhs.as_ref().0)
    }

    fn value(&self) -> Self::Type {
        self.0.value()
    }

    fn set_value(&mut self, value: Self::Type) {
        self.0.set_value(value)
    }
}

impl<W: Semiring> GallicWeight<W> {
    pub fn len(&self) -> usize {
        self.0.len()
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
    fn divide(&self, rhs: &Self, divide_type: DivideType) -> Self {
        Self(self.0.divide(&rhs.0, divide_type))
    }
}
