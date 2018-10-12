use semirings::Semiring;
use std::hash::{Hash, Hasher};
use Label;

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

    pub fn add_to_path(&mut self, input_label: Label, output_label: Label, weight: W) {
        self.ilabels.push(input_label);
        self.olabels.push(output_label);
        self.weight *= weight
    }

    pub fn add_weight(&mut self, weight: W) {
        self.weight *= weight
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

impl<W: Semiring + Hash + Eq> Hash for Path<W> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ilabels.hash(state);
        self.olabels.hash(state);
        self.weight.hash(state);
    }
}

impl<W: Semiring + Hash + Eq> Eq for Path<W> {}
