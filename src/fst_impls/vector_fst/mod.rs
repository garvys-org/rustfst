use arc::Arc;
use fst_traits::{
    ArcIterator, CoreFst, ExpandedFst, Fst, MutableArcIterator, MutableFst, StateIterator,
};
use semirings::Semiring;
use std::slice;
use Label;
use StateId;

#[derive(Debug, PartialEq)]
pub struct VectorFst<W: Semiring> {
    states: Vec<VectorFstState<W>>,
    start_state: Option<StateId>,
}

impl<W: 'static + Semiring> Fst for VectorFst<W> {}

impl<W: 'static + Semiring> CoreFst for VectorFst<W> {
    type W = W;
    fn start(&self) -> Option<StateId> {
        self.start_state
    }

    fn final_weight(&self, state_id: &StateId) -> Option<W> {
        if let Some(state) = self.states.get(*state_id) {
            state.final_weight.clone()
        } else {
            None
        }
    }

    fn num_arcs(&self) -> usize {
        self.states.iter().map(|state| state.num_arcs()).sum()
    }
}

impl<'a, W: 'a + Semiring> StateIterator<'a> for VectorFst<W> {
    type Iter = VectorStateIterator<'a, W>;
    // type Iter = Iterator<Item =&'a StateId>;
    fn states_iter(&'a self) -> Self::Iter {
        VectorStateIterator::new(self)
    }
}

pub struct VectorStateIterator<'a, W: 'a + Semiring> {
    fst: &'a VectorFst<W>,
    index: usize,
}

impl<'a, W: Semiring> VectorStateIterator<'a, W> {
    pub fn new(fst: &VectorFst<W>) -> VectorStateIterator<W> {
        VectorStateIterator { fst, index: 0 }
    }
}

impl<'a, W: Semiring> Iterator for VectorStateIterator<'a, W> {
    type Item = StateId;

    fn next(&mut self) -> Option<Self::Item> {
        let res = if self.index < self.fst.states.len() {
            Some(self.index)
        } else {
            None
        };
        self.index += 1;
        res
    }
}

impl<'a, W: 'static + Semiring> ArcIterator<'a> for VectorFst<W> {
    type Iter = slice::Iter<'a, Arc<W>>;
    fn arcs_iter(&'a self, state_id: &StateId) -> Self::Iter {
        self.states[*state_id].arcs.iter()
    }
}

impl<W: 'static + Semiring> ExpandedFst for VectorFst<W> {
    fn num_states(&self) -> usize {
        self.states.len()
    }
}

impl<W: 'static + Semiring> MutableFst for VectorFst<W> {
    fn new() -> Self {
        VectorFst {
            states: vec![],
            start_state: None,
        }
    }

    fn set_start(&mut self, state_id: &StateId) {
        assert!(self.states.get(*state_id).is_some());
        self.start_state = Some(*state_id);
    }

    fn set_final(&mut self, state_id: &StateId, final_weight: W) {
        if let Some(state) = self.states.get_mut(*state_id) {
            state.final_weight = Some(final_weight);
        } else {
            panic!("Stateid {:?} doesn't exist", state_id);
        }
    }

    fn add_state(&mut self) -> StateId {
        let id = self.states.len();
        self.states.insert(id, VectorFstState::default());
        id
    }

    fn add_arc(
        &mut self,
        source: &StateId,
        target: &StateId,
        ilabel: Label,
        olabel: Label,
        weight: W,
    ) {
        if let Some(state) = self.states.get_mut(*source) {
            state.arcs.push(Arc::new(ilabel, olabel, weight, *target));
        } else {
            panic!("State {:?} doesn't exist", source);
        }
    }

    fn del_state(&mut self, state_to_remove: &StateId) {
        // Remove the state from the vector
        // Check the arcs for arcs going to this state

        assert!(*state_to_remove < self.states.len());
        self.states.remove(*state_to_remove);
        for state in &mut self.states {
            let mut to_delete = vec![];
            for (arc_id, arc) in state.arcs.iter_mut().enumerate() {
                if arc.nextstate == *state_to_remove {
                    to_delete.push(arc_id);
                } else if arc.nextstate > *state_to_remove {
                    arc.nextstate -= 1;
                }
            }

            for id in to_delete.iter().rev() {
                state.arcs.remove(*id);
            }
        }
    }

    fn del_states<T: IntoIterator<Item = StateId>>(&mut self, states: T) {
        let mut v: Vec<_> = states.into_iter().collect();
        v.sort();
        for j in (0..v.len()).rev() {
            self.del_state(&v[j]);
        }
    }
}

impl<'a, W: 'static + Semiring> MutableArcIterator<'a> for VectorFst<W> {
    type IterMut = slice::IterMut<'a, Arc<W>>;
    fn arcs_iter_mut(&'a mut self, state_id: &StateId) -> Self::IterMut {
        self.states[*state_id].arcs.iter_mut()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VectorFstState<W: Semiring> {
    final_weight: Option<W>,
    arcs: Vec<Arc<W>>,
}

impl<W: Semiring> VectorFstState<W> {
    pub fn num_arcs(&self) -> usize {
        self.arcs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semirings::ProbabilityWeight;

    #[test]
    fn test_num_arcs() {
        let mut fst = VectorFst::new();
        let s1 = fst.add_state();
        let s2 = fst.add_state();

        assert_eq!(fst.num_arcs(), 0);
        fst.add_arc(&s1, &s2, 3, 5, ProbabilityWeight::new(10.0));
        assert_eq!(fst.num_arcs(), 1);
        fst.add_arc(&s1, &s2, 5, 7, ProbabilityWeight::new(18.0));
        assert_eq!(fst.num_arcs(), 2);
        fst.add_arc(&s2, &s1, 10, 17, ProbabilityWeight::new(38.0));
        assert_eq!(fst.num_arcs(), 3); 
        fst.add_arc(&s2, &s2, 10, 17, ProbabilityWeight::new(38.0));
        assert_eq!(fst.num_arcs(), 4);
        fst.del_state(&s1);
        assert_eq!(fst.num_arcs(), 1);
    }

    #[test]
    fn test_num_states() {
        let mut fst = VectorFst::<ProbabilityWeight>::new();
        assert_eq!(fst.num_states(), 0);
        
        let s1 = fst.add_state();
        assert_eq!(fst.num_states(), 1);
        
        fst.add_state();
        assert_eq!(fst.num_states(), 2);
        
        fst.del_state(&s1);
        assert_eq!(fst.num_states(), 1);

        fst.add_state();
        assert_eq!(fst.num_states(), 2);
    }

    #[test]
    fn test_arcs_iter() {
        let mut fst = VectorFst::new();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        fst.set_start(&s1);
        fst.add_arc(&s1, &s2, 3, 5, ProbabilityWeight::new(10.0));
        fst.add_arc(&s1, &s2, 5, 7, ProbabilityWeight::new(18.0));
        fst.add_arc(&s2, &s1, 10, 17, ProbabilityWeight::new(38.0));

        let mut arcs_s1 = fst.arcs_iter(&s1);

        let arc = arcs_s1.next().unwrap();
        assert_eq!(arc.ilabel, 3);
        assert_eq!(arc.olabel, 5);
        assert_eq!(arc.weight, ProbabilityWeight::new(10.0));
        assert_eq!(arc.nextstate, s2);

        let arc = arcs_s1.next().unwrap();
        assert_eq!(arc.ilabel, 5);
        assert_eq!(arc.olabel, 7);
        assert_eq!(arc.weight, ProbabilityWeight::new(18.0));
        assert_eq!(arc.nextstate, s2);

        assert!(arcs_s1.next().is_none());

        let mut arcs_s2 = fst.arcs_iter(&s2);

        let arc = arcs_s2.next().unwrap();
        assert_eq!(arc.ilabel, 10);
        assert_eq!(arc.olabel, 17);
        assert_eq!(arc.weight, ProbabilityWeight::new(38.0));
        assert_eq!(arc.nextstate, s1);

        assert!(arcs_s2.next().is_none());
    }

    #[test]
    fn test_arcs_iter_mut() {
        let mut fst = VectorFst::new();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        fst.set_start(&s1);
        fst.add_arc(&s1, &s2, 3, 5, ProbabilityWeight::new(10.0));
        fst.add_arc(&s1, &s2, 5, 7, ProbabilityWeight::new(18.0));

        for arc in fst.arcs_iter_mut(&s1) {
            println!("{:?}", arc);
            arc.ilabel = 53;
        }

        for arc in fst.arcs_iter(&s1) {
            println!("Pouet = {:?}", arc);
        }
    }
}
