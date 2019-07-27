use crate::fst_impls::VectorFst;
use crate::fst_traits::StateIterator;
use crate::semirings::Semiring;
use crate::StateId;

impl<'a, W: 'a + Semiring> StateIterator<'a> for VectorFst<W> {
    type Iter = VectorStateIterator<'a, W>;
    fn states_iter(&'a self) -> Self::Iter {
        VectorStateIterator::new(self)
    }
}

#[derive(Clone)]
pub struct VectorStateIterator<'a, W: 'a + Semiring> {
    fst: &'a VectorFst<W>,
    index: usize,
}

impl<'a, W: Semiring> VectorStateIterator<'a, W> {
    pub fn new(fst: &VectorFst<W>) -> VectorStateIterator<W> {
        VectorStateIterator { fst, index: 0 }
    }
}

impl<'a, W: Semiring> Iterator for VectorStateIterator<'a, W> {
    type Item = StateId;

    fn next(&mut self) -> Option<Self::Item> {
        let res = if self.index < self.fst.states.len() {
            Some(self.index)
        } else {
            None
        };
        self.index += 1;
        res
    }
}