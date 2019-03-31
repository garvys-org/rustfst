use crate::algorithms::factor_iterators::{
    StringFactorLeft, StringFactorRestrict, StringFactorRight,
};
use crate::algorithms::FactorIterator;
use crate::semirings::{
    GallicWeight, GallicWeightLeft, GallicWeightMin, GallicWeightRestrict, GallicWeightRight,
    Semiring, StringWeightVariant,
};

pub struct GallicFactorLeft<W: Semiring> {
    weight: GallicWeightLeft<W>,
    done: bool,
}

pub struct GallicFactorRight<W: Semiring> {
    weight: GallicWeightRight<W>,
    done: bool,
}

pub struct GallicFactorMin<W: Semiring> {
    weight: GallicWeightMin<W>,
    done: bool,
}

pub struct GallicFactorRestrict<W: Semiring> {
    weight: GallicWeightRestrict<W>,
    done: bool,
}

macro_rules! impl_gallic_factor {
    ($gallic: ident, $g_factor: ident, $s_factor: ident) => {
        impl<W: Semiring> Iterator for $g_factor<W> {
            type Item = ($gallic<W>, $gallic<W>);

            fn next(&mut self) -> Option<Self::Item> {
                let mut it = $s_factor::new(self.weight.value1().clone());
                let (p_f, p_s) = it.next().unwrap();
                let g1 = (p_f, self.weight.value2().clone()).into();
                let g2 = (p_s, W::one()).into();
                self.done = true;
                Some((g1, g2))
            }
        }

        impl<W: Semiring> FactorIterator<$gallic<W>> for $g_factor<W> {
            fn new(weight: $gallic<W>) -> Self {
                let done = match &weight.value1().value {
                    StringWeightVariant::Infinity => true,
                    StringWeightVariant::Labels(l) => (l.len() <= 1),
                };
                Self { weight, done }
            }

            fn done(&self) -> bool {
                self.done
            }
        }
    };
}

impl_gallic_factor!(GallicWeightLeft, GallicFactorLeft, StringFactorLeft);
impl_gallic_factor!(GallicWeightRight, GallicFactorRight, StringFactorRight);
impl_gallic_factor!(
    GallicWeightRestrict,
    GallicFactorRestrict,
    StringFactorRestrict
);
impl_gallic_factor!(GallicWeightMin, GallicFactorMin, StringFactorRestrict);

use std::slice::Iter as IterSlice;
pub struct GallicFactor<W: Semiring> {
    weight: GallicWeight<W>,
    done: bool,
    idx: usize,
}

impl<W: Semiring> Iterator for GallicFactor<W> {
    type Item = (GallicWeight<W>, GallicWeight<W>);

    fn next(&mut self) -> Option<Self::Item> {
        let w = &self.weight.0.list[self.idx];
        let mut s_it = StringFactorRestrict::new(w.value1().clone());
        let (p_f, p_s) = s_it.next().unwrap();

        let grw1: GallicWeightRestrict<W> = (p_f, w.value2().clone()).into();
        let grw2: GallicWeightRestrict<W> = (p_s, W::one()).into();
        self.idx += 1;
        Some((grw1.into(), grw2.into()))
    }
}

impl<W: Semiring> FactorIterator<GallicWeight<W>> for GallicFactor<W> {
    fn new(weight: GallicWeight<W>) -> Self {
        let mut done = false;
        if weight.0.list.len() == 0 {
            done = true;
        } else if weight.0.list.len() == 1 && weight.0.list[0].value1().len() <= 1 {
            done = true;
        }
        Self {
            weight,
            done,
            idx: 0,
        }
    }

    fn done(&self) -> bool {
        self.done || (self.idx >= self.weight.0.list.len())
    }
}
