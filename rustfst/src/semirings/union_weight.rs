use std::cmp::Ordering;
use std::fmt::Debug;
use std::fmt::{Display, Formatter, Result};
use std::hash::Hash;
use std::marker::PhantomData;

use failure::Fallible;

use crate::semirings::{
    DivideType, Semiring, SemiringProperties, WeaklyDivisibleSemiring, WeightQuantize,
};

pub trait UnionWeightOption<W: Semiring>: Debug + Hash + Default + Clone + PartialOrd + Eq {
    type ReverseOptions: UnionWeightOption<W::ReverseWeight>;
    fn compare(w1: &W, w2: &W) -> bool;
    fn merge(w1: &W, w2: &W) -> Fallible<W>;
}

/// Semiring that uses Times() and One() from W and union and the empty set
/// for Plus() and Zero(), respectively. Template argument O specifies the union
/// weight options as above.
#[derive(PartialOrd, PartialEq, Clone, Eq, Debug, Hash, Default)]
pub struct UnionWeight<W: Semiring, O: UnionWeightOption<W>> {
    pub(crate) list: Vec<W>,
    ghost: PhantomData<O>,
}

impl<W, O> Display for UnionWeight<W, O>
where
    W: Semiring,
    O: UnionWeightOption<W>,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        self.list.fmt(f)
    }
}

impl<W, O> AsRef<UnionWeight<W, O>> for UnionWeight<W, O>
where
    W: Semiring,
    O: UnionWeightOption<W>,
{
    fn as_ref(&self) -> &Self {
        &self
    }
}

impl<W: Semiring, O: UnionWeightOption<W>> Semiring for UnionWeight<W, O> {
    type Type = Vec<W>;
    type ReverseWeight = UnionWeight<W::ReverseWeight, O::ReverseOptions>;

    fn zero() -> Self {
        Self {
            list: vec![],
            ghost: PhantomData,
        }
    }

    fn one() -> Self {
        Self {
            list: vec![W::one()],
            ghost: PhantomData,
        }
    }

    fn new(value: Self::Type) -> Self {
        Self {
            list: value,
            ghost: PhantomData,
        }
    }

    fn plus_assign<P: AsRef<Self>>(&mut self, rhs: P) -> Fallible<()> {
        if self.is_zero() {
            self.set_value(rhs.as_ref().value());
        } else if rhs.as_ref().is_zero() {
            // Nothing
        } else {
            let mut sum: UnionWeight<W, O> = UnionWeight::zero();
            let n1 = self.list.len();
            let n2 = rhs.as_ref().list.len();
            let mut i1 = 0;
            let mut i2 = 0;
            while i1 < n1 && i2 < n2 {
                let v1 = unsafe { self.list.get_unchecked(i1) };
                let v2 = unsafe { rhs.as_ref().list.get_unchecked(i2) };
                if O::compare(v1, v2) {
                    sum.push_back(v1.clone(), true)?;
                    i1 += 1;
                } else {
                    sum.push_back(v2.clone(), true)?;
                    i2 += 1;
                }
            }

            for i in i1..n1 {
                let v1 = unsafe { self.list.get_unchecked(i) };
                sum.push_back(v1.clone(), true)?;
            }

            for i in i2..n2 {
                let v2 = unsafe { rhs.as_ref().list.get_unchecked(i) };
                sum.push_back(v2.clone(), true)?;
            }
            //TODO: Remove this copy and do the modification inplace
            self.set_value(sum.value());
        }
        Ok(())
    }

    fn times_assign<P: AsRef<Self>>(&mut self, rhs: P) -> Fallible<()> {
        if self.is_zero() || rhs.as_ref().is_zero() {
            self.set_value(Self::zero().value());
        } else {
            let mut prod1: UnionWeight<W, O> = UnionWeight::zero();
            for w1 in self.iter() {
                let mut prod2: UnionWeight<W, O> = UnionWeight::zero();
                for w2 in rhs.as_ref().iter() {
                    let p = w1.times(&w2)?;
                    prod2.push_back(p, true)?;
                }
                prod1.plus_assign(prod2)?;
            }
            self.set_value(prod1.value());
        }
        Ok(())
    }

    fn value(&self) -> Self::Type {
        self.list.clone()
    }

    fn set_value(&mut self, value: Self::Type) {
        self.list = value;
    }

    fn reverse(&self) -> Fallible<Self::ReverseWeight> {
        let mut rw = Self::ReverseWeight::zero();
        for v in self.iter() {
            rw.push_back(v.reverse()?, false)?;
        }
        rw.list.sort_by(|v1, v2| {
            if O::ReverseOptions::compare(v1, v2) {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });
        Ok(rw)
    }

    fn properties() -> SemiringProperties {
        W::properties()
            & (SemiringProperties::LEFT_SEMIRING
                | SemiringProperties::RIGHT_SEMIRING
                | SemiringProperties::COMMUTATIVE
                | SemiringProperties::IDEMPOTENT)
    }
}

impl<W: Semiring, O: UnionWeightOption<W>> UnionWeight<W, O> {
    fn push_back(&mut self, weight: W, sorted: bool) -> Fallible<()> {
        if self.list.is_empty() {
            self.list.push(weight);
        } else if sorted {
            let n = self.list.len();
            let back = &mut self.list[n - 1];
            if O::compare(back, &weight) {
                self.list.push(weight);
            } else {
                *back = O::merge(back, &weight)?;
            }
        } else {
            let first = &mut self.list[0];
            if O::compare(first, &weight) {
                self.list.push(weight);
            } else {
                let first_cloned = first.clone();
                *first = weight;
                self.list.push(first_cloned);
            }
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &W> {
        self.list.iter()
    }
}

impl<W, O> WeaklyDivisibleSemiring for UnionWeight<W, O>
where
    W: WeaklyDivisibleSemiring,
    O: UnionWeightOption<W>,
{
    fn divide(&self, rhs: &Self, divide_type: DivideType) -> Fallible<Self> {
        if self.is_zero() || rhs.is_zero() {
            return Ok(Self::zero());
        }
        let mut quot = Self::zero();
        if self.len() == 1 {
            for v in rhs.list.iter().rev() {
                quot.push_back(self.list[0].divide(v, divide_type)?, true)?;
            }
        } else if rhs.len() == 1 {
            for v in self.list.iter() {
                quot.push_back(v.divide(&rhs.list[0], divide_type)?, true)?;
            }
        } else {
            bail!("Expected at least of the two parameters to have a single element");
        }
        Ok(quot)
    }
}

impl<W, O> WeightQuantize for UnionWeight<W, O>
where
    W: WeightQuantize,
    O: UnionWeightOption<W>,
{
    fn quantize_assign(&mut self, delta: f32) -> Fallible<()> {
        let v: Vec<_> = self.list.drain(..).collect();
        for mut e in v {
            e.quantize_assign(delta)?;
            self.push_back(e.quantize(delta)?, true)?;
        }
        Ok(())
    }
}
