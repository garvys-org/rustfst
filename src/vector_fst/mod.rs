use fst::Fst;
use StateId;
use std::collections::HashMap;
use semirings::Semiring;
use arc::StdArc;

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
}

#[derive(Debug, Clone)]
pub struct VectorFstState<W: Semiring> {
    final_weight: Option<W>,
    arcs: Vec<StdArc<W>>,
}

impl<W: Semiring> VectorFstState<W> {
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