use failure::Fallible;

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

    pub fn is_empty_list(&self) -> bool {
        match self {
            StringWeightVariant::Infinity => false,
            StringWeightVariant::Labels(l) => l.len() == 0,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = StringWeightVariant> + '_ {
        StringWeightVariantIterator {
            v: &self,
            idx: 0,
            done: false,
        }
    }
}

impl Default for StringWeightVariant {
    fn default() -> Self {
        StringWeightVariant::Labels(vec![])
    }
}

impl From<Vec<Label>> for StringWeightVariant {
    fn from(l: Vec<usize>) -> Self {
        StringWeightVariant::Labels(l)
    }
}

struct StringWeightVariantIterator<'a> {
    v: &'a StringWeightVariant,
    idx: usize,
    done: bool,
}

impl<'a> Iterator for StringWeightVariantIterator<'a> {
    type Item = StringWeightVariant;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        match self.v {
            StringWeightVariant::Infinity => {
                self.done = true;
                Some(StringWeightVariant::Infinity)
            }
            StringWeightVariant::Labels(l) => {
                if self.idx < l.len() {
                    let res = Some(StringWeightVariant::Labels(vec![l[self.idx]]));
                    self.idx += 1;
                    res
                } else {
                    self.done = true;
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_string_variant_iterator_inf() -> Fallible<()> {
        let w = StringWeightVariant::Infinity;
        let mut it = w.iter();

        let n = it.next();
        assert_eq!(n, Some(StringWeightVariant::Infinity));

        let n = it.next();
        assert_eq!(n, None);

        Ok(())
    }

    #[test]
    fn test_string_variant_iterator_labels() -> Fallible<()> {
        let w = StringWeightVariant::Labels(vec![1, 2]);
        let mut it = w.iter();

        let n = it.next();
        assert_eq!(n, Some(StringWeightVariant::Labels(vec![1])));

        let n = it.next();
        assert_eq!(n, Some(StringWeightVariant::Labels(vec![2])));

        let n = it.next();
        assert_eq!(n, None);

        Ok(())
    }
}
