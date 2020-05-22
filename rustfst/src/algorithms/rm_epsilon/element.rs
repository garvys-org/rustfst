use crate::{Label, StateId};

#[derive(Hash, Debug, PartialOrd, PartialEq, Eq, Clone)]
pub struct Element {
    pub ilabel: Label,
    pub olabel: Label,
    pub nextstate: StateId,
}
