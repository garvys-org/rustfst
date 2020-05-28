use crate::algorithms::lazy_fst_revamp::FstCache;
use crate::{Semiring, TrsVec};
use std::ops::Deref;
use std::sync::Arc;

impl<W: Semiring, C: FstCache<W>> FstCache<W> for Arc<C> {
    fn get_start(&self) -> Option<Option<usize>> {
        self.deref().get_start()
    }

    fn insert_start(&self, id: Option<usize>) {
        self.deref().insert_start(id)
    }

    fn get_trs(&self, id: usize) -> Option<TrsVec<W>> {
        self.deref().get_trs(id)
    }

    fn insert_trs(&self, id: usize, trs: TrsVec<W>) {
        self.deref().insert_trs(id, trs)
    }

    fn get_final_weight(&self, id: usize) -> Option<Option<W>> {
        self.deref().get_final_weight(id)
    }

    fn insert_final_weight(&self, id: usize, weight: Option<W>) {
        self.deref().insert_final_weight(id, weight)
    }

    fn num_known_states(&self) -> usize {
        self.deref().num_known_states()
    }
}
