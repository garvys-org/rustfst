use arc::Arc;
use fst_traits::{
    ArcIterator, CoreFst, ExpandedFst, Fst, MutableArcIterator, MutableFst, StateIterator,
};
use semirings::Semiring;
use std::slice;
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

    fn add_arc(&mut self, source: &StateId, arc: Arc<<Self as CoreFst>::W>) {
        if let Some(state) = self.states.get_mut(*source) {
            state.arcs.push(arc);
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
    use fst_traits::FinalStatesIterator;
    use rand::rngs::StdRng;
    use rand::Rng;
    use rand::SeedableRng;
    use semirings::ProbabilityWeight;

    #[test]
    fn test_small_fst() {
        let mut fst = VectorFst::new();

        // States
        let s1 = fst.add_state();
        let s2 = fst.add_state();

        fst.set_start(&s1);

        // Arcs
        fst.add_arc(&s1, Arc::new(3, 5, ProbabilityWeight::new(10.0), s2));
        assert_eq!(fst.num_arcs(), 1);
        fst.add_arc(&s1, Arc::new(5, 7, ProbabilityWeight::new(18.0), s2));
        assert_eq!(fst.num_arcs(), 2);
        assert_eq!(fst.arcs_iter(&s1).count(), 2);

        // Iterates on arcs leaving s1
        let mut it_s1 = fst.arcs_iter(&s1);

        let a = it_s1.next();
        assert!(a.is_some());
        let a = a.unwrap();
        assert_eq!(a.ilabel, 3);
        assert_eq!(a.olabel, 5);
        assert_eq!(a.nextstate, s2);
        assert_eq!(a.weight, ProbabilityWeight::new(10.0));

        let b = it_s1.next();
        assert!(b.is_some());
        let b = b.unwrap();
        assert_eq!(b.ilabel, 5);
        assert_eq!(b.olabel, 7);
        assert_eq!(b.nextstate, s2);
        assert_eq!(b.weight, ProbabilityWeight::new(18.0));

        let c = it_s1.next();
        assert!(c.is_none());

        // Iterates on arcs leaving s2
        let mut it_s2 = fst.arcs_iter(&s2);

        let d = it_s2.next();
        assert!(d.is_none());
    }

    #[test]
    fn test_mutable_iter_arcs_small() {
        let mut fst = VectorFst::new();

        // States
        let s1 = fst.add_state();
        let s2 = fst.add_state();

        fst.set_start(&s1);

        // Arcs
        fst.add_arc(&s1, Arc::new(3, 5, ProbabilityWeight::new(10.0), s2));
        fst.add_arc(&s1, Arc::new(5, 7, ProbabilityWeight::new(18.0), s2));
    }

    #[test]
    fn test_start_states() {
        let mut fst = VectorFst::<ProbabilityWeight>::new();

        let n_states = 1000;

        // Add N states to the FST
        let states: Vec<_> = (0..n_states).map(|_| fst.add_state()).collect();

        // Should be no start state
        assert_eq!(fst.start(), None);

        // New start state
        fst.set_start(&states[18]);
        assert_eq!(fst.start(), Some(states[18]));

        // New start state
        fst.set_start(&states[32]);
        assert_eq!(fst.start(), Some(states[32]));
    }

    #[test]
    fn test_only_final_states() {
        let mut fst = VectorFst::<ProbabilityWeight>::new();

        let n_states = 1000;

        // Add N states to the FST
        let states: Vec<_> = (0..n_states).map(|_| fst.add_state()).collect();

        // Number of final states should be zero
        assert_eq!(fst.final_states_iter().count(), 0);

        // Set all states as final
        states
            .iter()
            .for_each(|v| fst.set_final(&v, ProbabilityWeight::one()));

        // Number of final states should be n_states
        assert_eq!(fst.final_states_iter().count(), n_states);
    }

    #[test]
    fn test_final_weight() {
        let mut fst = VectorFst::<ProbabilityWeight>::new();

        let n_states = 1000;
        let n_final_states = 300;

        // Add N states to the FST
        let mut states: Vec<_> = (0..n_states).map(|_| fst.add_state()).collect();

        // Nono the states are final => None final weight
        assert!(
            fst.states_iter()
                .map(|state_id| fst.final_weight(&state_id))
                .all(|v| v.is_none())
        );

        // Select randomly n_final_states
        let mut rg = StdRng::from_seed([53; 32]);
        rg.shuffle(&mut states);
        let final_states: Vec<_> = states.into_iter().take(n_final_states).collect();

        // Set those as final with a weight equals to its position in the vector
        final_states.iter().enumerate().for_each(|(idx, state_id)| {
            fst.set_final(state_id, ProbabilityWeight::new(idx as f32))
        });

        // Check they are final with the correct weight
        assert!(final_states.iter().all(|state_id| fst.is_final(state_id)));
        assert!(
            final_states
                .iter()
                .enumerate()
                .all(|(idx, state_id)| fst.final_weight(state_id)
                    == Some(ProbabilityWeight::new(idx as f32)))
        );
    }

    #[test]
    fn test_del_states_big() {
        let n_states = 1000;
        let n_states_to_delete = 300;

        let mut fst = VectorFst::<ProbabilityWeight>::new();

        // Add N states to the FST
        let mut states: Vec<_> = (0..n_states).map(|_| fst.add_state()).collect();

        // Check those N states do exist
        assert_eq!(fst.num_states(), n_states);

        // Sample n_states_to_delete to remove from the FST
        let mut rg = StdRng::from_seed([53; 32]);
        rg.shuffle(&mut states);
        let states_to_delete: Vec<_> = states.into_iter().take(n_states_to_delete).collect();

        fst.del_states(states_to_delete);

        // Check they are correctly removed
        assert_eq!(fst.num_states(), n_states - n_states_to_delete);
    }
}
