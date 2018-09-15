use fst::Fst;
use StateId;
use std::collections::HashMap;
use semirings::Semiring;
use arc::StdArc;

pub struct VectorFst<W: Semiring> {
	states: HashMap<StateId, VectorFstState<W>>,
	start_state: Option<StateId>,
}

pub struct VectorFstState<W: Semiring> {
	final_weight: Option<W>,
	arcs: Vec<StdArc<W>>,
}

impl<W: Semiring> Fst<W> for VectorFst<W> {
	type Arc= StdArc<W>;

	fn start(&self) -> Option<StateId> {
		self.start_state
	}

	fn final_weight(&self, state_id: &StateId) -> Option<W> {
		if let Some(state) = self.states.get(state_id) {
			state.final_weight.clone()
		}
		else {
			None
		}
	}

	fn is_final(&self, state_id: &StateId) -> bool {
		self.final_weight(state_id).is_some()
	}
}