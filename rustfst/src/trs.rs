use crate::semirings::Semiring;
use crate::Tr;
use std::fmt::Debug;
use std::sync::Arc;

pub trait Trs<W: Semiring>: std::ops::Deref<Target = [Tr<W>]> + Debug {
    fn trs(&self) -> &[Tr<W>];
    fn shallow_clone(&self) -> Self;
}

#[derive(Debug, PartialOrd, PartialEq)]
pub struct TrsVec<W: Semiring>(pub(crate) Arc<Vec<Tr<W>>>);

impl<W: Semiring> Trs<W> for TrsVec<W> {
    fn trs(&self) -> &[Tr<W>] {
        self.0.as_slice()
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
