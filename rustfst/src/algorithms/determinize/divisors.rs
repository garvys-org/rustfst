use std::fmt::Debug;

use anyhow::Result;

use crate::semirings::{
    GallicWeight, GallicWeightLeft, GallicWeightMin, GallicWeightRestrict, StringWeightLeft,
    StringWeightRestrict,
};
use crate::Semiring;

pub trait CommonDivisor<W: Semiring>: PartialEq + Debug + Sync {
    fn common_divisor(w1: &W, w2: &W) -> Result<W>;
}

#[derive(PartialEq, Debug)]
pub struct DefaultCommonDivisor {}

impl<W: Semiring> CommonDivisor<W> for DefaultCommonDivisor {
    fn common_divisor(w1: &W, w2: &W) -> Result<W> {
        w1.plus(w2)
    }
}

#[derive(PartialEq, Debug)]
pub struct LabelCommonDivisor {}

macro_rules! impl_label_common_divisor {
    ($string_semiring: ident) => {
        impl CommonDivisor<$string_semiring> for LabelCommonDivisor {
            fn common_divisor(
                w1: &$string_semiring,
                w2: &$string_semiring,
            ) -> Result<$string_semiring> {
                let mut iter1 = w1.iter();
                let mut iter2 = w2.iter();
                if w1.value.is_empty_list() || w2.value.is_empty_list() {
                    Ok($string_semiring::one())
                } else if w1.value.is_infinity() {
                    Ok(iter2.next().unwrap().into())
                } else if w2.value.is_infinity() {
                    Ok(iter1.next().unwrap().into())
                } else {
                    let v1 = iter1.next().unwrap();
                    let v2 = iter2.next().unwrap();
                    if v1 == v2 {
                        Ok(v1.into())
                    } else {
                        Ok($string_semiring::one())
                    }
                }
            }
        }
    };
}

impl_label_common_divisor!(StringWeightLeft);
impl_label_common_divisor!(StringWeightRestrict);

#[derive(Debug, PartialEq)]
pub struct GallicCommonDivisor {}

macro_rules! impl_gallic_common_divisor {
    ($gallic: ident) => {
        impl<W: Semiring> CommonDivisor<$gallic<W>> for GallicCommonDivisor {
            fn common_divisor(w1: &$gallic<W>, w2: &$gallic<W>) -> Result<$gallic<W>> {
                let v1 = LabelCommonDivisor::common_divisor(w1.value1(), w2.value1())?;
                let v2 = DefaultCommonDivisor::common_divisor(w1.value2(), w2.value2())?;
                Ok((v1, v2).into())
            }
        }
    };
}

impl_gallic_common_divisor!(GallicWeightLeft);
impl_gallic_common_divisor!(GallicWeightRestrict);
impl_gallic_common_divisor!(GallicWeightMin);

impl<W: Semiring> CommonDivisor<GallicWeight<W>> for GallicCommonDivisor {
    fn common_divisor(w1: &GallicWeight<W>, w2: &GallicWeight<W>) -> Result<GallicWeight<W>> {
        let mut weight = GallicWeightRestrict::zero();
        for w in w1.iter().chain(w2.iter()) {
            weight = GallicCommonDivisor::common_divisor(&weight, w)?;
        }
        if weight.is_zero() {
            Ok(GallicWeight::zero())
        } else {
            Ok(weight.into())
        }
    }
}
