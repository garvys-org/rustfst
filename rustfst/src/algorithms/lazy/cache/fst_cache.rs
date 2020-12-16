use std::fmt::Debug;
use std::ops::Deref;

use crate::algorithms::lazy::CacheStatus;
use crate::semirings::Semiring;
use crate::{StateId, TrsVec};

pub trait FstCache<W: Semiring>: Debug {
    fn get_start(&self) -> CacheStatus<Option<StateId>>;
    fn insert_start(&self, id: Option<StateId>);

    fn get_trs(&self, id: StateId) -> CacheStatus<TrsVec<W>>;
    fn insert_trs(&self, id: StateId, trs: TrsVec<W>);

    fn get_final_weight(&self, id: StateId) -> CacheStatus<Option<W>>;
    fn insert_final_weight(&self, id: StateId, weight: Option<W>);

    fn num_known_states(&self) -> usize;
    fn compute_num_known_trs(&self) -> usize;

    fn num_trs(&self, id: StateId) -> Option<usize>;

    fn num_input_epsilons(&self, id: StateId) -> Option<usize>;
    fn num_output_epsilons(&self, id: StateId) -> Option<usize>;

    fn len_trs(&self) -> usize;
    fn len_final_weights(&self) -> usize;
}

impl<W: Semiring, C: FstCache<W>, CP: Deref<Target = C> + Debug> FstCache<W> for CP {
    fn get_start(&self) -> CacheStatus<Option<StateId>> {
        self.deref().get_start()
    }

    fn insert_start(&self, id: Option<StateId>) {
        self.deref().insert_start(id)
    }

    fn get_trs(&self, id: StateId) -> CacheStatus<TrsVec<W>> {
        self.deref().get_trs(id)
    }

    fn insert_trs(&self, id: StateId, trs: TrsVec<W>) {
        self.deref().insert_trs(id, trs)
    }

    fn get_final_weight(&self, id: StateId) -> CacheStatus<Option<W>> {
        self.deref().get_final_weight(id)
    }

    fn insert_final_weight(&self, id: StateId, weight: Option<W>) {
        self.deref().insert_final_weight(id, weight)
    }

    fn num_known_states(&self) -> usize {
        self.deref().num_known_states()
    }

    fn compute_num_known_trs(&self) -> usize {
        self.deref().compute_num_known_trs()
    }

    fn num_trs(&self, id: StateId) -> Option<usize> {
        self.deref().num_trs(id)
    }

    fn num_input_epsilons(&self, id: StateId) -> Option<usize> {
        self.deref().num_input_epsilons(id)
    }

    fn num_output_epsilons(&self, id: StateId) -> Option<usize> {
        self.deref().num_output_epsilons(id)
    }

    fn len_trs(&self) -> usize {
        self.deref().len_trs()
    }

    fn len_final_weights(&self) -> usize {
        self.deref().len_final_weights()
    }
}
