use Label;
use StateId;
use semirings::Semiring;

#[derive(Debug, Clone)]
pub struct Arc<W: Semiring> {
    pub ilabel: Label,
    pub olabel: Label,
    pub weight: W,
    pub nextstate: StateId,
}

impl<W: Semiring> Arc<W> {
    pub fn new(ilabel: Label, olabel: Label, weight: W, nextstate: StateId) -> Self {
        Arc {
            ilabel, olabel, weight, nextstate
        }
    }
}
