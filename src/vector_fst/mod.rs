use fst::{Fst, ExpandedFst, MutableFst};
use StateId;
use std::collections::HashMap;
use semirings::Semiring;
use arc::StdArc;
use Label;

#[derive(Debug)]
pub struct VectorFst<W: Semiring> {
    states: HashMap<StateId, VectorFstState<W>>,
    start_state: Option<StateId>,
}

impl<W: Semiring> Fst<W> for VectorFst<W> {
    type Arc = StdArc<W>;
    type Iter = VectorArcIterator<W>;

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

    fn arc_iter(&self, state_id: &StateId) -> Self::Iter {
        VectorArcIterator {state : self.states[state_id].clone(), arcindex: 0}
    }

    fn num_arcs(&self) -> usize {
        self.states.iter().map(|(_, state)| state.num_arcs()).sum()
    }
}

impl<W: Semiring> ExpandedFst<W> for VectorFst<W> {
    fn num_states(&self) -> usize {
        self.states.len()
    }
}

impl<W: Semiring> MutableFst<W> for VectorFst<W> {
    fn new() -> Self {
        VectorFst {
            states: HashMap::new(),
            start_state: None,
        }
    }

    fn set_start(&mut self, state_id: &StateId) {
        assert!(self.states.get(state_id).is_some());
        self.start_state = Some(*state_id);
    }

    fn set_final(&mut self, state_id: &StateId, final_weight: W) {
        if let Some(state) = self.states.get_mut(state_id) {
            state.final_weight = Some(final_weight);
        }
        else {
            panic!("Stateid {:?} doesn't exist", state_id);
        }
    }

    fn add_state(&mut self) -> StateId {
        let id = self.states.len();
        self.states.insert(id, VectorFstState::new());
        id
    }

    fn add_arc(&mut self, source: StateId, target: StateId, ilabel: Label, olabel: Label, weight: W) {
        if let Some(state) = self.states.get_mut(&source) {
            state.arcs.push(StdArc::new(ilabel, olabel, weight, target));
        }
        else {
            panic!("State {:?} doesn't exist", source);
        }
    }
}

#[derive(Debug, Clone)]
pub struct VectorFstState<W: Semiring> {
    final_weight: Option<W>,
    arcs: Vec<StdArc<W>>,
}

impl<W: Semiring> VectorFstState<W> {
    pub fn new() -> Self {
        VectorFstState {
            final_weight: None,
            arcs: vec![],
        }
    }

    pub fn num_arcs(&self) -> usize {
        self.arcs.len()
    }
}

#[derive(Debug)]
pub struct VectorArcIterator<W: Semiring> {
    state: VectorFstState<W>,
    arcindex: usize,
}

impl<W: Semiring> Iterator for VectorArcIterator<W> {
    type Item = StdArc<W>;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.arcindex;
        let res = if i < self.state.num_arcs() {
            Some(self.state.arcs[i].clone())
        } else {
            None
        };
        self.arcindex += 1;
        res
    }
}