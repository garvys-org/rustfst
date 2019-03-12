use std::fmt;

use crate::semirings::Semiring;
use crate::Label;

#[derive(Clone, Debug, PartialOrd, PartialEq, Eq, Hash)]
pub enum StringWeightVariant {
    Infinity,
    Labels(Vec<Label>),
}

impl StringWeightVariant {
    pub fn unwrap_labels(&self) -> &Vec<Label> {
        match self {
            StringWeightVariant::Infinity => panic!("lol"),
            StringWeightVariant::Labels(l) => l,
        }
    }
}

impl Default for StringWeightVariant {
    fn default() -> Self {
        StringWeightVariant::Labels(vec![])
    }
}

#[derive(Clone, Debug, PartialOrd, Default, PartialEq, Eq, Hash)]
pub struct StringWeightRestrict {
    /// If None -> Infinity. If Some([]) -> Epsilon
    pub value: StringWeightVariant,
}

#[derive(Clone, Debug, PartialOrd, Default, PartialEq, Eq, Hash)]
pub struct StringWeightLeft {
    /// If None -> Infinity. If Some([]) -> Epsilon
    pub value: StringWeightVariant,
}

#[derive(Clone, Debug, PartialOrd, Default, PartialEq, Eq, Hash)]
pub struct StringWeightRight {
    /// If None -> Infinity. If Some([]) -> Epsilon
    pub value: StringWeightVariant,
}

pub enum StringType {
    StringRestrict,
    StringLeft,
    StringRight,
}

macro_rules! string_semiring {
    ($semiring: ty, $string_type: expr) => {
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
                            // FIXME
                            write!(f, "{:?}", v)?;
                        }
                    }
                };
                Ok(())
            }
        }

        impl AsRef<Self> for $semiring {
            fn as_ref(&self) -> &$semiring {
                &self
            }
        }

        impl Semiring for $semiring {
            type Type = StringWeightVariant;

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

            fn plus_assign<P: AsRef<Self>>(&mut self, rhs: P) {
                if self.is_zero() {
                    self.set_value(rhs.as_ref().value());
                } else if rhs.as_ref().is_zero() {
                    // Do nothing
                } else {
                    let l1 = self.value.unwrap_labels();
                    let l2 = rhs.as_ref().value.unwrap_labels();

                    match $string_type {
                        StringType::StringRestrict => {
                            if self != rhs.as_ref() {
                                panic!(
                                    "Unequal arguments : non-functional FST ? w1 = {:?} w2 = {:?}",
                                    &self,
                                    &rhs.as_ref()
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
                    }
                }
            }
            fn times_assign<P: AsRef<Self>>(&mut self, rhs: P) {
                if let StringWeightVariant::Labels(ref mut labels_left) = self.value {
                    if let StringWeightVariant::Labels(ref labels_right) = rhs.as_ref().value {
                        for l in labels_right {
                            labels_left.push(*l);
                        }
                    } else {
                        self.value = StringWeightVariant::Infinity;
                    }
                }
            }

            fn value(&self) -> <Self as Semiring>::Type {
                self.value.clone()
            }

            fn set_value(&mut self, value: <Self as Semiring>::Type) {
                self.value = value;
            }
        }

        impl $semiring {
            pub fn len(&self) -> usize {
                match &self.value {
                    StringWeightVariant::Infinity => 0,
                    StringWeightVariant::Labels(l) => l.len(),
                }
            }
        }
    };
}

string_semiring!(StringWeightRestrict, StringType::StringRestrict);
string_semiring!(StringWeightLeft, StringType::StringLeft);
string_semiring!(StringWeightRight, StringType::StringRight);
