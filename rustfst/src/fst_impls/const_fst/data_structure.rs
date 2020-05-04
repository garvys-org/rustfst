use crate::{StateId, SymbolTable, Tr};
use std::rc::Rc;

/// Immutable FST whose states and trs each implemented by single arrays,
#[derive(Debug, PartialEq, Clone)]
pub struct ConstFst<W> {
    pub(crate) states: Vec<ConstState<W>>,
    pub(crate) trs: Vec<Tr<W>>,
    pub(crate) start: Option<StateId>,
    pub(crate) isymt: Option<Rc<SymbolTable>>,
    pub(crate) osymt: Option<Rc<SymbolTable>>,
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
