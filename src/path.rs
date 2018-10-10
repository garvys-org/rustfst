use semirings::Semiring;
use Label;

#[derive(PartialEq, Debug)]
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
