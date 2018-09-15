use Label;
use StateId;
use semirings::Semiring;

pub trait Arc<W: Semiring>  {
    fn ilabel(&self) -> Label;
    fn olabel(&self) -> Label;
    fn weight(&self) -> W;
    fn nextstate(&self) -> StateId;
}

#[derive(Debug, Clone)]
pub struct StdArc<W: Semiring> {
	ilabel: Label,
	olabel: Label,
	weight: W,
	nextstate: StateId,
}

impl<W: Semiring> StdArc<W> {
	pub fn new(ilabel: Label, olabel: Label, weight: W, nextstate: StateId) -> Self {
		StdArc {
			ilabel, olabel, weight, nextstate
		}
	}
}

impl<W: Semiring> Arc<W> for StdArc<W> {
	fn ilabel(&self) -> Label {
		self.ilabel
	}
	fn olabel(&self) -> Label {
		self.olabel
	}
	fn weight(&self) -> W {
		self.weight.clone()
	}
	fn nextstate(&self) -> StateId {
		self.nextstate
	}
}