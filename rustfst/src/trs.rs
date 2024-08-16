use crate::semirings::Semiring;
use crate::Tr;
use std::fmt::Debug;
use std::sync::Arc;

pub trait Trs<W: Semiring>: std::ops::Deref<Target = [Tr<W>]> + Debug {
    fn trs(&self) -> &[Tr<W>];
    fn to_trs_vec(&self) -> TrsVec<W>;
    fn shallow_clone(&self) -> Self;
}

#[derive(Debug, PartialOrd, PartialEq, Eq)]
pub struct TrsVec<W: Semiring>(pub Arc<Vec<Tr<W>>>);

impl<W: Semiring> Trs<W> for TrsVec<W> {
    fn trs(&self) -> &[Tr<W>] {
        self.0.as_slice()
    }

    fn to_trs_vec(&self) -> TrsVec<W> {
        self.shallow_clone()
    }

    fn shallow_clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<W: Semiring> TrsVec<W> {
    pub fn remove(&mut self, index: usize) -> Tr<W> {
        Arc::make_mut(&mut self.0).remove(index)
    }
    pub fn push(&mut self, tr: Tr<W>) {
        Arc::make_mut(&mut self.0).push(tr)
    }

    pub fn clear(&mut self) {
        Arc::make_mut(&mut self.0).clear()
    }
}

impl<W: Semiring> Clone for TrsVec<W> {
    fn clone(&self) -> Self {
        Self(Arc::new((*self.0).clone()))
    }
}

impl<W: Semiring> std::ops::Deref for TrsVec<W> {
    type Target = [Tr<W>];
    fn deref(&self) -> &Self::Target {
        self.trs()
    }
}

impl<W: Semiring> Default for TrsVec<W> {
    fn default() -> Self {
        Self(Arc::new(vec![]))
    }
}

impl<W: Semiring> From<Vec<Tr<W>>> for TrsVec<W> {
    fn from(v: Vec<Tr<W>>) -> Self {
        Self(Arc::new(v))
    }
}

#[derive(Debug, PartialOrd, PartialEq)]
pub struct TrsConst<W: Semiring> {
    pub(crate) trs: Arc<Vec<Tr<W>>>,
    pub(crate) pos: usize,
    pub(crate) n: usize,
}

impl<W: Semiring> Trs<W> for TrsConst<W> {
    fn trs(&self) -> &[Tr<W>] {
        &self.trs[self.pos..self.pos + self.n]
    }

    fn to_trs_vec(&self) -> TrsVec<W> {
        TrsVec(Arc::new(self.trs().to_vec()))
    }

    // Doesn't clone the data, only the Arc
    fn shallow_clone(&self) -> Self {
        Self {
            trs: Arc::clone(&self.trs),
            pos: self.pos,
            n: self.n,
        }
    }
}

impl<W: Semiring> Clone for TrsConst<W> {
    fn clone(&self) -> Self {
        Self {
            trs: Arc::new((*self.trs).clone()),
            n: self.n,
            pos: self.pos,
        }
    }
}

impl<W: Semiring> std::ops::Deref for TrsConst<W> {
    type Target = [Tr<W>];
    fn deref(&self) -> &Self::Target {
        self.trs()
    }
}

impl<W: Semiring> Default for TrsConst<W> {
    fn default() -> Self {
        Self {
            trs: Arc::new(vec![]),
            pos: 0,
            n: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_trs_const {
        use super::*;
        use crate::prelude::TropicalWeight;
        use anyhow::Result;

        #[test]
        fn test_to_trs_vec() -> Result<()> {
            let trs = TrsConst {
                trs: Arc::new(vec![
                    Tr::<TropicalWeight>::new(1, 1, TropicalWeight::one(), 0),
                    Tr::<TropicalWeight>::new(1, 1, TropicalWeight::one(), 0),
                ]),
                pos: 1,
                n: 1,
            };

            let tr_vec = trs.to_trs_vec();
            assert_eq!(tr_vec.len(), 1);

            Ok(())
        }
    }
}
