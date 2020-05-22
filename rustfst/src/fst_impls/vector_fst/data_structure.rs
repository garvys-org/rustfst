use std::sync::Arc;

use crate::algorithms::tr_filters::TrFilter;
use crate::algorithms::tr_filters::{InputEpsilonTrFilter, OutputEpsilonTrFilter};
use crate::semirings::Semiring;
use crate::symbol_table::SymbolTable;
use crate::{StateId, TrsVec};

/// Simple concrete, mutable FST whose states and trs are stored in standard vectors.
///
/// All states are stored in a vector of states.
/// In each state, there is a vector of trs containing the outgoing transitions.
#[derive(Debug, PartialEq, Clone)]
pub struct VectorFst<W: Semiring> {
    pub(crate) states: Vec<VectorFstState<W>>,
    pub(crate) start_state: Option<StateId>,
    pub(crate) isymt: Option<Arc<SymbolTable>>,
    pub(crate) osymt: Option<Arc<SymbolTable>>,
}

// In my opinion, it is not a good idea to store values like num_trs, num_input_epsilons
// and num_output_epsilons inside the data structure as it would mean having to maintain them
// when the object is modified. Which is not trivial with the MutableTrIterator API for instance.
// Same goes for TrMap. For not-mutable fst however, it is usefull.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorFstState<W: Semiring> {
    pub(crate) final_weight: Option<W>,
    pub(crate) trs: TrsVec<W>,
}

impl<W: Semiring> VectorFstState<W> {
    pub fn new() -> Self {
        Self {
            final_weight: None,
            trs: TrsVec::default(),
        }
    }
    pub fn num_trs(&self) -> usize {
        self.trs.len()
    }
}

impl<W: Semiring> VectorFstState<W> {
    pub fn num_input_epsilons(&self) -> usize {
        let filter = InputEpsilonTrFilter {};
        self.trs.iter().filter(|v| filter.keep(v)).count()
    }

    pub fn num_output_epsilons(&self) -> usize {
        let filter = OutputEpsilonTrFilter {};
        self.trs.iter().filter(|v| filter.keep(v)).count()
    }
}
