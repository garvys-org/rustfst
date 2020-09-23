use std::fmt::Debug;

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
    fn num_trs(&self, id: StateId) -> Option<usize>;

    fn num_input_epsilons(&self, id: usize) -> CacheStatus<usize>;
    unsafe fn num_input_epsilons_unchecked(&self, id: usize) -> usize;

    fn num_output_epsilons(&self, id: usize) -> CacheStatus<usize>;
    unsafe fn num_output_epsilons_unchecked(&self, id: usize) -> usize;

    fn len_trs(&self) -> usize;
    fn len_final_weights(&self) -> usize;

    fn is_final(&self, state_id: StateId) -> CacheStatus<bool>;
    unsafe fn is_final_unchecked(&self, state_id: StateId) -> bool;
}
