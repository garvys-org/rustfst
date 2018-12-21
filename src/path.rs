use std::hash::{Hash, Hasher};

use crate::semirings::Semiring;
use crate::{Label, EPS_LABEL};

#[derive(PartialEq, Debug, Clone, PartialOrd)]
pub struct Path<W: Semiring> {
    pub ilabels: Vec<Label>,
    pub olabels: Vec<Label>,
    pub weight: W,
}

impl<W: Semiring> Path<W> {
    pub fn new(ilabels: Vec<Label>, olabels: Vec<Label>, weight: W) -> Self {
        Path {
            ilabels,
            olabels,
            weight,
        }
    }

    pub fn add_to_path(&mut self, ilabel: Label, olabel: Label, weight: W) {
        if ilabel != EPS_LABEL {
            self.ilabels.push(ilabel);
        }

        if olabel != EPS_LABEL {
            self.olabels.push(olabel);
        }

        self.weight *= weight
    }

    pub fn add_weight(&mut self, weight: W) {
        self.weight *= weight
    }

    pub fn concat(&mut self, other: Path<W>) {
        self.ilabels.extend(other.ilabels);
        self.olabels.extend(other.olabels);
        self.weight *= other.weight;
    }
}

impl<W: Semiring> Default for Path<W> {
    fn default() -> Self {
        Path {
            ilabels: vec![],
            olabels: vec![],
            weight: W::one(),
        }
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl<W: Semiring + Hash + Eq> Hash for Path<W> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ilabels.hash(state);
        self.olabels.hash(state);
        self.weight.hash(state);
    }
}

impl<W: Semiring + Hash + Eq> Eq for Path<W> {}
