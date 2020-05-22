use crate::{Label, Semiring, StateId, Tr};

#[derive(PartialEq, Eq, Clone, Hash, PartialOrd, Debug)]
pub struct DeterminizeElement<W: Semiring> {
    pub state: StateId,
    pub weight: W,
}

impl<W: Semiring> DeterminizeElement<W> {
    pub fn new(state: StateId, weight: W) -> Self {
        DeterminizeElement { state, weight }
    }
}

#[derive(Default, PartialEq, Eq, Clone, Hash, PartialOrd, Debug)]
pub struct WeightedSubset<W: Semiring> {
    pub pairs: Vec<DeterminizeElement<W>>,
}

impl<W: Semiring> WeightedSubset<W> {
    pub fn from_vec(vec: Vec<DeterminizeElement<W>>) -> Self {
        WeightedSubset { pairs: vec }
    }

    pub fn iter(&self) -> impl Iterator<Item = &DeterminizeElement<W>> {
        self.pairs.iter()
    }
}

#[derive(Default, PartialEq, Eq, Clone, Hash, PartialOrd, Debug)]
pub struct DeterminizeStateTuple<W: Semiring> {
    pub subset: WeightedSubset<W>,
    pub filter_state: StateId,
}

#[derive(Default, PartialEq, Eq, Clone, Hash, PartialOrd, Debug)]
pub struct DeterminizeTr<W: Semiring> {
    pub label: Label,
    pub weight: W,
    pub dest_tuple: DeterminizeStateTuple<W>,
}

impl<W: Semiring> DeterminizeTr<W> {
    pub fn from_tr(tr: &Tr<W>, filter_state: StateId) -> Self {
        Self {
            label: tr.ilabel,
            weight: W::zero(),
            dest_tuple: DeterminizeStateTuple {
                subset: WeightedSubset::from_vec(vec![]),
                filter_state,
            },
        }
    }
}
