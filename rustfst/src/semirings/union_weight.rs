use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::io::Write;
use std::marker::PhantomData;

use anyhow::Result;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::multi::{count, separated_list0};
use nom::IResult;

use crate::parsers::nom_utils::NomCustomError;
use crate::parsers::parse_bin_i32;
use crate::parsers::write_bin_i32;
use crate::semirings::{
    DivideType, ReverseBack, Semiring, SemiringProperties, SerializableSemiring,
    WeaklyDivisibleSemiring, WeightQuantize,
};

pub trait UnionWeightOption<W: Semiring>:
    Debug + Hash + Clone + PartialOrd + Eq + Sync + 'static
{
    type ReverseOptions: UnionWeightOption<W::ReverseWeight>;
    fn compare(w1: &W, w2: &W) -> bool;
    fn merge(w1: &W, w2: &W) -> Result<W>;
}

/// Semiring that uses Times() and One() from W and union and the empty set
/// for Plus() and Zero(), respectively. Template argument O specifies the union
/// weight options as above.
#[derive(PartialOrd, PartialEq, Clone, Eq, Debug, Hash, Default)]
pub struct UnionWeight<W: Semiring, O: UnionWeightOption<W>> {
    pub(crate) list: Vec<W>,
    ghost: PhantomData<O>,
}

impl<W, O> AsRef<UnionWeight<W, O>> for UnionWeight<W, O>
where
    W: Semiring,
    O: UnionWeightOption<W>,
{
    fn as_ref(&self) -> &Self {
        self
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

    fn plus_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Result<()> {
        if self.is_zero() {
            self.set_value(rhs.borrow().value().clone());
        } else if rhs.borrow().is_zero() {
            // Nothing
        } else {
            let mut sum: UnionWeight<W, O> = UnionWeight::zero();
            let n1 = self.list.len();
            let n2 = rhs.borrow().list.len();
            let mut i1 = 0;
            let mut i2 = 0;
            while i1 < n1 && i2 < n2 {
                let v1 = unsafe { self.list.get_unchecked(i1) };
                let v2 = unsafe { rhs.borrow().list.get_unchecked(i2) };
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
                let v2 = unsafe { rhs.borrow().list.get_unchecked(i) };
                sum.push_back(v2.clone(), true)?;
            }
            //TODO: Remove this copy and do the modification inplace
            self.set_value(sum.take_value());
        }
        Ok(())
    }

    fn times_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Result<()> {
        if self.is_zero() || rhs.borrow().is_zero() {
            self.set_value(Self::zero().take_value());
        } else {
            let mut prod1: UnionWeight<W, O> = UnionWeight::zero();
            for w1 in self.iter() {
                let mut prod2: UnionWeight<W, O> = UnionWeight::zero();
                for w2 in rhs.borrow().iter() {
                    let p = w1.times(w2)?;
                    prod2.push_back(p, true)?;
                }
                prod1.plus_assign(prod2)?;
            }
            self.set_value(prod1.take_value());
        }
        Ok(())
    }

    fn approx_equal<P: Borrow<Self>>(&self, rhs: P, delta: f32) -> bool {
        if self.len() != rhs.borrow().len() {
            return false;
        }
        let it1 = self.iter();
        let it2 = rhs.borrow().iter();
        for (w1, w2) in it1.zip(it2) {
            if !w1.approx_equal(w2, delta) {
                return false;
            }
        }
        true
    }

    fn value(&self) -> &Self::Type {
        &self.list
    }

    fn take_value(self) -> Self::Type {
        self.list
    }

    fn set_value(&mut self, value: Self::Type) {
        self.list = value;
    }

    fn reverse(&self) -> Result<Self::ReverseWeight> {
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
    fn push_back(&mut self, weight: W, sorted: bool) -> Result<()> {
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
    fn divide_assign(&mut self, rhs: &Self, divide_type: DivideType) -> Result<()> {
        if self.is_zero() || rhs.is_zero() {
            self.list.clear();
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
        self.set_value(quot.take_value());
        Ok(())
    }
}

impl<W, O> WeightQuantize for UnionWeight<W, O>
where
    W: WeightQuantize,
    O: UnionWeightOption<W>,
{
    fn quantize_assign(&mut self, delta: f32) -> Result<()> {
        let v: Vec<_> = self.list.drain(..).collect();
        for mut e in v {
            e.quantize_assign(delta)?;
            self.push_back(e.quantize(delta)?, true)?;
        }
        Ok(())
    }
}

impl<W, O> UnionWeight<W, O>
where
    W: SerializableSemiring,
    O: UnionWeightOption<W>,
{
    fn parse_text_empty_set(i: &str) -> IResult<&str, Self> {
        let (i, _) = tag("EmptySet")(i)?;
        Ok((i, Self::zero()))
    }

    fn parse_text_non_empty_set(i: &str) -> IResult<&str, Self> {
        let (i, weights) = separated_list0(tag(","), W::parse_text)(i)?;
        Ok((i, Self::new(weights)))
    }
}

impl<W, O> Display for UnionWeight<W, O>
where
    W: SerializableSemiring,
    O: UnionWeightOption<W>,
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "EmptySet")?;
        } else {
            for (idx, w) in self.list.iter().enumerate() {
                if idx > 0 {
                    write!(f, ",")?;
                }
                write!(f, "{}", w)?;
            }
        }
        Ok(())
    }
}

impl<W, O> SerializableSemiring for UnionWeight<W, O>
where
    W: SerializableSemiring,
    O: UnionWeightOption<W>,
{
    fn weight_type() -> String {
        format!("{}_union", W::weight_type())
    }

    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>> {
        let (i, n) = parse_bin_i32(i)?;
        let (i, labels) = count(W::parse_binary, n as usize)(i)?;
        Ok((i, Self::new(labels)))
    }

    fn write_binary<F: Write>(&self, file: &mut F) -> Result<()> {
        write_bin_i32(file, self.list.len() as i32)?;
        for w in self.list.iter() {
            w.write_binary(file)?;
        }
        Ok(())
    }

    fn parse_text(i: &str) -> IResult<&str, Self> {
        let (i, res) = alt((Self::parse_text_empty_set, Self::parse_text_non_empty_set))(i)?;
        Ok((i, res))
    }
}

impl<W: Semiring, O: UnionWeightOption<W>> ReverseBack<UnionWeight<W, O>>
    for <UnionWeight<W, O> as Semiring>::ReverseWeight
{
    fn reverse_back(&self) -> Result<UnionWeight<W, O>> {
        let res = self.reverse()?;
        // TODO: Find a way to avoid this transmute. For the moment, it is necessary because of the compare function.
        unsafe { Ok(std::mem::transmute(res)) }
    }
}
