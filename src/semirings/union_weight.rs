use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use crate::semirings::{DivideType, Semiring, WeaklyDivisibleSemiring, WeightQuantize};

pub trait UnionWeightOption<W: Semiring>: Debug + Hash + Default + Clone + PartialOrd + Eq {
    fn compare(w1: &W, w2: &W) -> bool;
    fn merge(w1: &W, w2: &W) -> W;
}

/// Semiring that uses Times() and One() from W and union and the empty set
/// for Plus() and Zero(), respectively. Template argument O specifies the union
/// weight options as above.
#[derive(PartialOrd, PartialEq, Clone, Eq, Debug, Hash, Default)]
pub struct UnionWeight<W: Semiring, O: UnionWeightOption<W>> {
    list: Vec<W>,
    ghost: PhantomData<O>,
}

impl<W, O> std::fmt::Display for UnionWeight<W, O>
where
    W: Semiring,
    O: UnionWeightOption<W>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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

    fn plus_assign<P: AsRef<Self>>(&mut self, rhs: P) {
        if self.is_zero() {
            self.set_value(rhs.as_ref().value());
        } else if rhs.as_ref().is_zero() {
            // Nothing
        } else {
            let mut sum: UnionWeight<W, O> = UnionWeight::zero();
            let n1 = self.list.len();
            let n2 = self.list.len();
            let n = n1.min(n2);
            for i in 0..n {
                let v1 = unsafe { self.list.get_unchecked(i) };
                let v2 = unsafe { rhs.as_ref().list.get_unchecked(i) };
                if O::compare(v1, v2) {
                    sum.push_back(v1.clone(), true);
                } else {
                    sum.push_back(v2.clone(), true);
                }
            }

            for i in n..n1 {
                let v1 = unsafe { self.list.get_unchecked(i) };
                sum.push_back(v1.clone(), true);
            }

            for i in n..n2 {
                let v2 = unsafe { rhs.as_ref().list.get_unchecked(i) };
                sum.push_back(v2.clone(), true);
            }
            //TODO: Remove this copy and do the modification inplace
            self.set_value(sum.value());
        }
    }

    fn times_assign<P: AsRef<Self>>(&mut self, rhs: P) {
        if self.is_zero() || rhs.as_ref().is_zero() {
            self.set_value(Self::zero().value());
        } else {
            let mut prod1: UnionWeight<W, O> = UnionWeight::zero();
            let n1 = self.list.len();
            let n2 = rhs.as_ref().list.len();
            for i in 0..n1 {
                let v1 = unsafe { self.list.get_unchecked(i) };
                let mut prod2: UnionWeight<W, O> = UnionWeight::zero();
                for j in 0..n2 {
                    let v2 = unsafe { rhs.as_ref().list.get_unchecked(j) };
                    prod2.push_back(v1.times(&v2), true);
                }
                prod1.plus_assign(prod2);
            }
            self.set_value(prod1.value());
        }
    }

    fn value(&self) -> Self::Type {
        self.list.clone()
    }

    fn set_value(&mut self, value: Self::Type) {
        self.list = value;
    }
}

impl<W: Semiring, O: UnionWeightOption<W>> UnionWeight<W, O> {
    fn push_back(&mut self, weight: W, sorted: bool) {
        if self.list.is_empty() {
            self.list.push(weight);
        } else if sorted {
            let n = self.list.len();
            let back = self.list.get_mut(n - 1).unwrap();
            if O::compare(back, &weight) {
                self.list.push(weight);
            } else {
                *back = O::merge(back, &weight);
            }
        } else {
            let first = self.list.get_mut(0).unwrap();
            if O::compare(first, &weight) {
                self.list.push(weight);
            } else {
                let first_cloned = first.clone();
                *first = weight;
                self.list.push(first_cloned);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }
}

impl<W, O> WeaklyDivisibleSemiring for UnionWeight<W, O>
where
    W: WeaklyDivisibleSemiring,
    O: UnionWeightOption<W>,
{
    fn divide(&self, rhs: &Self, divide_type: DivideType) -> Self {
        if self.is_zero() || rhs.is_zero() {
            return Self::zero();
        }
        let mut quot = Self::one();
        if self.len() == 1 {
            for v in rhs.list.iter().rev() {
                quot.push_back(self.list[0].divide(v, divide_type), true);
            }
        } else if rhs.len() == 1 {
            for v in self.list.iter() {
                quot.push_back(rhs.list[0].divide(v, divide_type), true);
            }
        } else {
            panic!("lol");
        }
        quot
    }
}

impl<W, O> WeightQuantize for UnionWeight<W, O>
where
    W: WeightQuantize,
    O: UnionWeightOption<W>,
{
    fn quantize_assign(&mut self, delta: f32) {
        let v: Vec<_> = self.list.drain(..).collect();
        for mut e in v {
            e.quantize_assign(delta);
            self.push_back(e.quantize(delta), true);
        }
    }
}
