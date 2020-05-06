use std::sync::Arc;
use crate::Tr;
use crate::semirings::Semiring;

pub trait Trs<W> : std::ops::Deref<Target=[Tr<W>]> {
    fn trs(&self) -> &[Tr<W>];
    fn shallow_clone(&self) -> Self;
}

#[derive(Debug, PartialOrd, PartialEq)]
pub struct TrsVec<W>(Arc<Vec<Tr<W>>>);

impl<W> Trs<W> for TrsVec<W> {
    fn trs(&self) -> &[Tr<W>] {
        self.0.as_slice()
    }

    fn shallow_clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<W: Clone> Clone for TrsVec<W> {
    fn clone(&self) -> Self {
        Self(Arc::new((*self.0).clone()))
    }
}

impl<W> std::ops::Deref for TrsVec<W> {
    type Target=[Tr<W>];
    fn deref(&self) -> &Self::Target {
        self.trs()
    }
}

impl<W> Default for TrsVec<W> {
    fn default() -> Self {
        Self(Arc::new(vec![]))
    }
}

#[derive(Debug, PartialOrd, PartialEq)]
pub struct TrsConst<W>{
    pub(crate) trs: Arc<Vec<Tr<W>>>,
    pub(crate) pos: usize,
    pub(crate) n: usize
}

impl<W> Trs<W> for TrsConst<W> {
    fn trs(&self) -> &[Tr<W>] {
        &self.trs[self.pos..self.pos+self.n]
    }

    // Doesn't clone the data, only the Arc
    fn shallow_clone(&self) -> Self {
        Self {
            trs: Arc::clone(&self.trs),
            pos: self.pos,
            n: self.n
        }
    }
}

impl<W: Clone> Clone for TrsConst<W> {
    fn clone(&self) -> Self {
        Self {
            trs: Arc::new((*self.trs).clone()),
            n: self.n,
            pos: self.pos
        }
    }
}

impl<W> std::ops::Deref for TrsConst<W> {
    type Target=[Tr<W>];
    fn deref(&self) -> &Self::Target {
        self.trs()
    }
}

impl<W> Default for TrsConst<W> {
    fn default() -> Self {
        Self{
            trs: Arc::new(vec![]),
            pos: 0,
            n: 0
        }
    }
}