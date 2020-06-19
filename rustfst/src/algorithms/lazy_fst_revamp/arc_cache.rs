use std::ops::Deref;
use std::sync::Arc;

use crate::algorithms::lazy_fst_revamp::FstCache;
use crate::{Semiring, TrsVec};

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

    fn num_trs(&self, id: usize) -> Option<usize> {
        self.deref().num_trs(id)
    }

    fn num_input_epsilons(&self, id: usize) -> Option<usize> {
        self.deref().num_input_epsilons(id)
    }

    fn num_output_epsilons(&self, id: usize) -> Option<usize> {
        self.deref().num_output_epsilons(id)
    }

    fn len_trs(&self) -> usize {
        self.deref().len_trs()
    }

    fn len_final_weights(&self) -> usize {
        self.deref().len_final_weights()
    }
}
