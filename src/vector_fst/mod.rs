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

    fn add_arc(&mut self, source: &StateId, target: &StateId, ilabel: Label, olabel: Label, weight: W) {
        if let Some(state) = self.states.get_mut(&source) {
            state.arcs.push(StdArc::new(ilabel, olabel, weight, *target));
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

#[cfg(test)]
mod tests {
    use super::*;
    use semirings::integer_weight::IntegerWeight;
    use arc::Arc;

    #[test]
    fn test_1() {
        let mut fst = VectorFst::new();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        fst.set_start(&s1);
        fst.add_arc(&s1, &s2, 3, 5, IntegerWeight::new(10));
        fst.add_arc(&s1, &s2, 5, 7, IntegerWeight::new(18));

        assert_eq!(fst.num_states(), 2);
        assert_eq!(fst.num_arcs(), 2);
        assert_eq!(fst.arc_iter(&s1).count(), 2);

        let mut it = fst.arc_iter(&s1);

        let a = it.next();
        assert!(a.is_some());
        let a = a.unwrap();
        assert_eq!(a.ilabel(), 3);
        assert_eq!(a.olabel(), 5);
        assert_eq!(a.nextstate(), s2);
        assert_eq!(a.weight(), IntegerWeight::new(10));

        let b = it.next();
        assert!(b.is_some());
        let b = b.unwrap();
        assert_eq!(b.ilabel(), 5);
        assert_eq!(b.olabel(), 7);
        assert_eq!(b.nextstate(), s2);
        assert_eq!(b.weight(), IntegerWeight::new(18));

        let c = it.next();
        assert!(c.is_none());
        // assert!(!it.done());
    }
}
