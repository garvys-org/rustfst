use std::borrow::Borrow;
use std::fmt;
use std::io::Write;

use anyhow::Result;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::multi::{count, separated_list0};
use nom::IResult;

use crate::parsers::nom_utils::{num, NomCustomError};
use crate::parsers::parse_bin_i32;
use crate::parsers::write_bin_i32;
use crate::semirings::string_variant::StringWeightVariant;
use crate::semirings::{
    DivideType, ReverseBack, Semiring, SemiringProperties, SerializableSemiring,
    WeaklyDivisibleSemiring, WeightQuantize,
};
use crate::Label;

/// String semiring: (identity, ., Infinity, Epsilon)
#[derive(Clone, Debug, PartialOrd, PartialEq, Eq, Hash)]
pub struct StringWeightRestrict {
    pub(crate) value: StringWeightVariant,
}

/// String semiring: (longest_common_prefix, ., Infinity, Epsilon)
#[derive(Clone, Debug, PartialOrd, PartialEq, Eq, Hash)]
pub struct StringWeightLeft {
    pub(crate) value: StringWeightVariant,
}

/// String semiring: (longest_common_suffix, ., Infinity, Epsilon)
#[derive(Clone, Debug, PartialOrd, PartialEq, Eq, Hash)]
pub struct StringWeightRight {
    pub(crate) value: StringWeightVariant,
}

/// Determines whether to use left or right string semiring.  Includes a
/// 'restricted' version that signals an error if proper prefixes/suffixes
/// would otherwise be returned by Plus, useful with various
/// algorithms that require functional transducer input with the
/// string semirings.
pub enum StringType {
    StringRestrict,
    StringLeft,
    StringRight,
}

macro_rules! string_semiring {
    ($semiring: ty, $string_type: expr, $reverse_semiring: ty) => {
        impl AsRef<Self> for $semiring {
            fn as_ref(&self) -> &$semiring {
                &self
            }
        }

        impl ReverseBack<$semiring> for <$semiring as Semiring>::ReverseWeight {
            fn reverse_back(&self) -> Result<$semiring> {
                self.reverse()
            }
        }

        impl Semiring for $semiring {
            type Type = StringWeightVariant;
            type ReverseWeight = $reverse_semiring;

            fn zero() -> Self {
                Self {
                    value: StringWeightVariant::Infinity,
                }
            }

            fn one() -> Self {
                Self {
                    value: StringWeightVariant::Labels(vec![]),
                }
            }

            fn new(value: <Self as Semiring>::Type) -> Self {
                Self { value }
            }

            fn plus_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Result<()> {
                if self.is_zero() {
                    self.set_value(rhs.borrow().value().clone());
                } else if rhs.borrow().is_zero() {
                    // Do nothing
                } else {
                    let l1 = self.value.unwrap_labels();
                    let l2 = rhs.borrow().value.unwrap_labels();

                    match $string_type {
                        StringType::StringRestrict => {
                            if self != rhs.borrow() {
                                bail!(
                                    "Unequal arguments : non-functional FST ? w1 = {:?} w2 = {:?}",
                                    &self,
                                    &rhs.borrow()
                                );
                            }
                        }
                        StringType::StringLeft => {
                            let new_labels: Vec<_> = l1
                                .iter()
                                .zip(l2.iter())
                                .take_while(|(v1, v2)| v1 == v2)
                                .map(|(v1, _)| v1)
                                .cloned()
                                .collect();
                            self.value = StringWeightVariant::Labels(new_labels);
                        }
                        StringType::StringRight => {
                            let new_labels: Vec<_> = l1
                                .iter()
                                .rev()
                                .zip(l2.iter().rev())
                                .take_while(|(v1, v2)| v1 == v2)
                                .map(|(v1, _)| v1)
                                .cloned()
                                .collect();
                            let new_labels: Vec<_> = new_labels.into_iter().rev().collect();
                            self.value = StringWeightVariant::Labels(new_labels);
                        }
                    };
                };
                Ok(())
            }
            fn times_assign<P: Borrow<Self>>(&mut self, rhs: P) -> Result<()> {
                if let StringWeightVariant::Labels(ref mut labels_left) = self.value {
                    if let StringWeightVariant::Labels(ref labels_right) = rhs.borrow().value {
                        for l in labels_right {
                            labels_left.push(*l);
                        }
                    } else {
                        self.value = StringWeightVariant::Infinity;
                    }
                }
                Ok(())
            }

            fn approx_equal<P: Borrow<Self>>(&self, rhs: P, _delta: f32) -> bool {
                self.value() == rhs.borrow().value()
            }

            fn value(&self) -> &<Self as Semiring>::Type {
                &self.value
            }

            fn take_value(self) -> <Self as Semiring>::Type {
                self.value
            }

            fn set_value(&mut self, value: <Self as Semiring>::Type) {
                self.value = value;
            }

            fn reverse(&self) -> Result<Self::ReverseWeight> {
                Ok(self.value().reverse().into())
            }

            fn properties() -> SemiringProperties {
                match $string_type {
                    StringType::StringRestrict => {
                        SemiringProperties::LEFT_SEMIRING
                            | SemiringProperties::RIGHT_SEMIRING
                            | SemiringProperties::IDEMPOTENT
                    }
                    StringType::StringLeft => {
                        SemiringProperties::LEFT_SEMIRING | SemiringProperties::IDEMPOTENT
                    }
                    StringType::StringRight => {
                        SemiringProperties::RIGHT_SEMIRING | SemiringProperties::IDEMPOTENT
                    }
                }
            }
        }

        impl $semiring {
            pub fn len_labels(&self) -> usize {
                match &self.value {
                    StringWeightVariant::Infinity => 0,
                    StringWeightVariant::Labels(l) => l.len(),
                }
            }

            pub fn iter(&self) -> impl Iterator<Item = StringWeightVariant> + '_ {
                self.value.iter()
            }
        }

        impl From<Vec<Label>> for $semiring {
            fn from(l: Vec<Label>) -> Self {
                Self::new(l.into())
            }
        }

        impl From<Label> for $semiring {
            fn from(l: Label) -> Self {
                Self::new(vec![l].into())
            }
        }

        impl From<StringWeightVariant> for $semiring {
            fn from(v: StringWeightVariant) -> Self {
                Self::new(v)
            }
        }

        impl WeightQuantize for $semiring {
            fn quantize_assign(&mut self, _delta: f32) -> Result<()> {
                // Nothing to do
                Ok(())
            }
        }

        impl fmt::Display for $semiring {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match &self.value {
                    StringWeightVariant::Infinity => {
                        write!(f, "Infinity")?;
                    }
                    StringWeightVariant::Labels(v) => {
                        if v.is_empty() {
                            write!(f, "Epsilon")?;
                        } else {
                            for (idx, label) in v.iter().enumerate() {
                                if idx > 0 {
                                    write!(f, "_")?;
                                }
                                write!(f, "{}", label)?;
                            }
                        }
                    }
                };
                Ok(())
            }
        }

        impl $semiring {
            fn parse_text_infinity(i: &str) -> IResult<&str, Self> {
                let (i, _) = tag("Infinity")(i)?;
                Ok((i, <$semiring>::new(StringWeightVariant::Infinity)))
            }

            fn parse_text_espilon(i: &str) -> IResult<&str, Self> {
                let (i, _) = tag("Epsilon")(i)?;
                Ok((i, <$semiring>::new(StringWeightVariant::Labels(vec![]))))
            }

            fn parse_text_labels(i: &str) -> IResult<&str, Self> {
                let (i, labels) = separated_list0(tag("_"), num)(i)?;
                Ok((i, <$semiring>::new(StringWeightVariant::Labels(labels))))
            }
        }

        impl SerializableSemiring for $semiring {
            fn weight_type() -> String {
                match $string_type {
                    StringType::StringRestrict => "restricted_string".to_string(),
                    StringType::StringLeft => "left_string".to_string(),
                    StringType::StringRight => "right_string".to_string(),
                }
            }

            fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>> {
                let (i, n) = parse_bin_i32(i)?;
                let (i, labels) = count(parse_bin_i32, n as usize)(i)?;
                // Check for infinity
                let weight = if labels == vec![-1] {
                    Self::new(StringWeightVariant::Infinity)
                } else {
                    Self::new(StringWeightVariant::Labels(
                        labels.into_iter().map(|e| e as Label).collect(),
                    ))
                };
                Ok((i, weight))
            }

            fn write_binary<F: Write>(&self, file: &mut F) -> Result<()> {
                match &self.value {
                    StringWeightVariant::Infinity => {
                        write_bin_i32(file, 1 as i32)?;
                        write_bin_i32(file, -1 as i32)?;
                    }
                    StringWeightVariant::Labels(labels) => {
                        write_bin_i32(file, labels.len() as i32)?;
                        for label in labels.iter() {
                            write_bin_i32(file, *label as i32)?;
                        }
                    }
                }
                Ok(())
            }

            fn parse_text(i: &str) -> IResult<&str, Self> {
                let (i, res) = alt((
                    Self::parse_text_infinity,
                    Self::parse_text_espilon,
                    Self::parse_text_labels,
                ))(i)?;
                Ok((i, res))
            }
        }
    };
}

string_semiring!(
    StringWeightRestrict,
    StringType::StringRestrict,
    StringWeightRestrict
);
string_semiring!(StringWeightLeft, StringType::StringLeft, StringWeightRight);
string_semiring!(StringWeightRight, StringType::StringRight, StringWeightLeft);

fn divide_left(w1: &StringWeightVariant, w2: &StringWeightVariant) -> StringWeightVariant {
    match (w1, w2) {
        (StringWeightVariant::Infinity, StringWeightVariant::Infinity) => panic!("Unexpected"),
        (StringWeightVariant::Infinity, StringWeightVariant::Labels(_)) => {
            StringWeightVariant::Infinity
        }
        (StringWeightVariant::Labels(_), StringWeightVariant::Infinity) => panic!("Unexpected"),
        (StringWeightVariant::Labels(l1), StringWeightVariant::Labels(l2)) => {
            StringWeightVariant::Labels(l1.iter().skip(l2.len()).cloned().collect())
        }
    }
}

fn divide_right(w1: &StringWeightVariant, w2: &StringWeightVariant) -> StringWeightVariant {
    match (w1, w2) {
        (StringWeightVariant::Infinity, StringWeightVariant::Infinity) => panic!("Unexpected"),
        (StringWeightVariant::Infinity, StringWeightVariant::Labels(_)) => {
            StringWeightVariant::Infinity
        }
        (StringWeightVariant::Labels(_), StringWeightVariant::Infinity) => panic!("Unexpected"),
        (StringWeightVariant::Labels(l1), StringWeightVariant::Labels(l2)) => {
            StringWeightVariant::Labels(l1.iter().rev().skip(l2.len()).rev().cloned().collect())
        }
    }
}

impl WeaklyDivisibleSemiring for StringWeightLeft {
    fn divide_assign(&mut self, rhs: &Self, divide_type: DivideType) -> Result<()> {
        if divide_type != DivideType::DivideLeft {
            bail!("Only left division is defined.");
        }
        self.value = divide_left(&self.value, &rhs.value);
        Ok(())
    }
}

impl WeaklyDivisibleSemiring for StringWeightRight {
    fn divide_assign(&mut self, rhs: &Self, divide_type: DivideType) -> Result<()> {
        if divide_type != DivideType::DivideRight {
            bail!("Only right division is defined.");
        }
        self.value = divide_right(&self.value, &rhs.value);
        Ok(())
    }
}

impl WeaklyDivisibleSemiring for StringWeightRestrict {
    fn divide_assign(&mut self, rhs: &Self, divide_type: DivideType) -> Result<()> {
        self.value = match divide_type {
            DivideType::DivideLeft => divide_left(&self.value, &rhs.value),
            DivideType::DivideRight => divide_right(&self.value, &rhs.value),
            DivideType::DivideAny => bail!("Only explicit left or right division is defined."),
        };
        Ok(())
    }
}

test_semiring_serializable!(
    tests_string_weight_left_serializable,
    StringWeightLeft,
    StringWeightLeft::one() StringWeightLeft::zero()
    StringWeightLeft::new(StringWeightVariant::Labels(vec![1]))
    StringWeightLeft::new(StringWeightVariant::Labels(vec![4, 5, 2]))
);

test_semiring_serializable!(
    tests_string_weight_right_serializable,
    StringWeightRight,
    StringWeightRight::one() StringWeightRight::zero()
    StringWeightRight::new(StringWeightVariant::Labels(vec![1]))
    StringWeightRight::new(StringWeightVariant::Labels(vec![4, 5, 2]))
);

test_semiring_serializable!(
    tests_string_weight_restrict_serializable,
    StringWeightRestrict,
    StringWeightRestrict::one() StringWeightRestrict::zero()
    StringWeightRestrict::new(StringWeightVariant::Labels(vec![1]))
    StringWeightRestrict::new(StringWeightVariant::Labels(vec![4, 5, 2]))
);
