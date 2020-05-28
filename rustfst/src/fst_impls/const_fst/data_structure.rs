use std::sync::Arc;

use crate::fst_properties::properties::EXPANDED;
use crate::fst_properties::FstProperties;
use crate::{Semiring, StateId, SymbolTable, Tr};

/// Immutable FST whose states and trs each implemented by single arrays,
#[derive(Debug, Clone)]
pub struct ConstFst<W> {
    pub(crate) states: Vec<ConstState<W>>,
    pub(crate) trs: Arc<Vec<Tr<W>>>,
    pub(crate) start: Option<StateId>,
    pub(crate) isymt: Option<Arc<SymbolTable>>,
    pub(crate) osymt: Option<Arc<SymbolTable>>,
    pub(crate) properties: FstProperties,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConstState<W> {
    /// Final Weight
    pub(crate) final_weight: Option<W>,
    /// Start of state's trs in `trs`.
    pub(crate) pos: usize,
    /// Number of trs (per state).
    pub(crate) ntrs: usize,
    /// Number of input epsilons
    pub(crate) niepsilons: usize,
    /// Number of output epsilons
    pub(crate) noepsilons: usize,
}

impl<W: Semiring> ConstFst<W> {
    pub(crate) fn static_properties() -> u64 {
        EXPANDED
    }
}

impl<W: Semiring> PartialEq for ConstFst<W> {
    fn eq(&self, other: &Self) -> bool {
        // Indended: Doesn't check symt and properties
        self.states == other.states && self.trs == other.trs && self.start == other.start
    }
}
