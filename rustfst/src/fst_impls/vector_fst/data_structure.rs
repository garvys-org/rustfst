use std::sync::Arc;

use crate::fst_properties::mutable_properties::add_tr_properties;
use crate::fst_properties::properties::{EXPANDED, MUTABLE};
use crate::fst_properties::FstProperties;
use crate::fst_traits::CoreFst;
use crate::semirings::Semiring;
use crate::symbol_table::SymbolTable;
use crate::{StateId, Tr, Trs, TrsVec, EPS_LABEL};

/// Simple concrete, mutable FST whose states and trs are stored in standard vectors.
///
/// All states are stored in a vector of states.
/// In each state, there is a vector of trs containing the outgoing transitions.
#[derive(Debug, Clone)]
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
    pub(crate) niepsilons: usize,
    pub(crate) noepsilons: usize,
}

impl<W: Semiring> PartialEq for VectorFst<W> {
    fn eq(&self, other: &Self) -> bool {
        // Indended: Doesn't check properties and symbol tables.
        self.states == other.states && self.start_state == other.start_state
    }
}

impl<W: Semiring> Default for VectorFstState<W> {
    fn default() -> Self {
        Self {
            final_weight: None,
            trs: TrsVec::default(),
            niepsilons: 0,
            noepsilons: 0,
        }
    }
}

impl<W: Semiring> VectorFstState<W> {
    pub fn new() -> Self {
        Self {
            final_weight: None,
            trs: TrsVec::default(),
            niepsilons: 0,
            noepsilons: 0,
        }
    }
    pub fn num_trs(&self) -> usize {
        self.trs.len()
    }
}

impl<W: Semiring> VectorFstState<W> {
    pub fn increment_num_epsilons(&mut self, tr: &Tr<W>) {
        if tr.ilabel == EPS_LABEL {
            self.niepsilons += 1;
        }
        if tr.olabel == EPS_LABEL {
            self.noepsilons += 1;
        }
    }
}

impl<W: Semiring> VectorFst<W> {
    pub fn update_properties_after_add_tr(&mut self, state: StateId) {
        let vector_state = unsafe { self.states.get_unchecked(state as usize) };

        // Safe because at least one Tr has been added
        let new_tr = vector_state.trs.last().unwrap();
        let old_tr = if vector_state.trs.trs().len() > 1 {
            Some(&vector_state.trs.trs()[vector_state.trs.trs().len() - 2])
        } else {
            None
        };
        self.properties = add_tr_properties(self.properties(), state, new_tr, old_tr);
    }

    pub fn static_properties() -> u64 {
        EXPANDED | MUTABLE
    }
}
