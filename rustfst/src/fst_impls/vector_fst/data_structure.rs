use std::sync::Arc;

use crate::algorithms::tr_filters::TrFilter;
use crate::algorithms::tr_filters::{InputEpsilonTrFilter, OutputEpsilonTrFilter};
use crate::fst_properties::mutable_properties::add_tr_properties;
use crate::fst_properties::properties::{EXPANDED, MUTABLE};
use crate::fst_properties::FstProperties;
use crate::semirings::Semiring;
use crate::symbol_table::SymbolTable;
use crate::{StateId, Trs, TrsVec};

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
    pub(crate) properties: FstProperties,
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

impl<W: Semiring> VectorFst<W> {
    pub fn proto_properties(&self) -> FstProperties {
        self.properties
    }

    pub fn proto_properties_2(&self, mask: FstProperties) -> FstProperties {
        self.properties & mask
    }

    pub fn update_properties_after_add_tr(&mut self, state: StateId) {
        let vector_state = unsafe { self.states.get_unchecked(state) };

        // Safe because at least one Tr has been added
        let new_tr = vector_state.trs.last().unwrap();
        let old_tr = if vector_state.trs.trs().len() > 1 {
            Some(&vector_state.trs.trs()[vector_state.trs.trs().len() - 2])
        } else {
            None
        };
        self.properties = add_tr_properties(self.properties, state, new_tr, old_tr);
    }

    pub fn static_properties() -> u64 {
        EXPANDED | MUTABLE
    }
}
