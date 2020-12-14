use crate::algorithms::lazy::StateTable;
use crate::{Label, StateId};

#[derive(Hash, Eq, PartialOrd, PartialEq, Clone, Debug)]
pub struct PrefixTuple {
    pub fst_id: Option<Label>,
    pub nextstate: Option<StateId>,
}

#[derive(Hash, Eq, PartialOrd, PartialEq, Clone, Debug)]
pub struct ReplaceStackPrefix {
    prefix: Vec<PrefixTuple>,
}

impl ReplaceStackPrefix {
    pub fn new() -> Self {
        Self { prefix: vec![] }
    }

    pub fn push(&mut self, fst_id: Option<StateId>, nextstate: Option<StateId>) {
        self.prefix.push(PrefixTuple { fst_id, nextstate });
    }

    pub fn pop(&mut self) {
        self.prefix.pop();
    }

    pub fn top(&self) -> &PrefixTuple {
        self.prefix.last().as_ref().unwrap()
    }
}

#[derive(Hash, Eq, PartialOrd, PartialEq, Clone, Debug)]
pub struct ReplaceStateTuple {
    /// Index in prefix table.
    pub prefix_id: StateId,
    /// Current FST being walked.
    pub fst_id: Option<StateId>,
    /// Current state in FST being walked (not to be
    /// confused with the thse StateId of the combined FST).
    pub fst_state: Option<StateId>,
}

impl ReplaceStateTuple {
    pub fn new(prefix_id: StateId, fst_id: Option<StateId>, fst_state: Option<StateId>) -> Self {
        Self {
            prefix_id,
            fst_id,
            fst_state,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ReplaceStateTable {
    pub prefix_table: StateTable<ReplaceStackPrefix>,
    pub tuple_table: StateTable<ReplaceStateTuple>,
}

impl ReplaceStateTable {
    pub fn new() -> Self {
        Self {
            prefix_table: StateTable::new(),
            tuple_table: StateTable::new(),
        }
    }
}
