use std::fmt::Debug;

use crate::semirings::Semiring;
use crate::{StateId, TrsVec};

pub trait FstCache<W: Semiring>: Debug {
    fn get_start(&self) -> Option<Option<StateId>>;
    fn insert_start(&self, id: Option<StateId>);

    fn get_trs(&self, id: StateId) -> Option<TrsVec<W>>;
    fn insert_trs(&self, id: StateId, trs: TrsVec<W>);

    fn get_final_weight(&self, id: StateId) -> Option<Option<W>>;
    fn insert_final_weight(&self, id: StateId, weight: Option<W>);

    fn num_known_states(&self) -> usize;
}
