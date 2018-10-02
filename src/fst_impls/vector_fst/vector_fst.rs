use arc::Arc;
use fst_traits::{
    ArcIterator, CoreFst, ExpandedFst, Fst, MutableArcIterator, MutableFst, StateIterator,
};
use semirings::Semiring;
use std::slice;
use Result;
use StateId;

#[derive(Debug, PartialEq, Clone)]
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
    fn arcs_iter(&'a self, state_id: &StateId) -> Result<Self::Iter> {
        let state = self
            .states
            .get(*state_id)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        Ok(state.arcs.iter())
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

    fn set_start(&mut self, state_id: &StateId) -> Result<()> {
        ensure!(
            self.states.get(*state_id).is_some(),
            "The state {:?} doesn't exist",
            state_id
        );
        self.start_state = Some(*state_id);
        Ok(())
    }

    fn set_final(&mut self, state_id: &StateId, final_weight: W) -> Result<()> {
        if let Some(state) = self.states.get_mut(*state_id) {
            state.final_weight = Some(final_weight);
            Ok(())
        } else {
            bail!("Stateid {:?} doesn't exist", state_id);
        }
    }

    fn add_state(&mut self) -> StateId {
        let id = self.states.len();
        self.states.insert(id, VectorFstState::default());
        id
    }

    fn add_arc(&mut self, source: &StateId, arc: Arc<<Self as CoreFst>::W>) -> Result<()> {
        if let Some(state) = self.states.get_mut(*source) {
            state.arcs.push(arc);
            Ok(())
        } else {
            bail!("State {:?} doesn't exist", source);
        }
    }

    fn del_state(&mut self, state_to_remove: &StateId) -> Result<()> {
        // Remove the state from the vector
        // Check the arcs for arcs going to this state

        ensure!(
            *state_to_remove < self.states.len(),
            "State id {:?} doesn't exist",
            *state_to_remove
        );
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
        Ok(())
    }

    fn del_states<T: IntoIterator<Item = StateId>>(&mut self, states: T) -> Result<()> {
        let mut v: Vec<_> = states.into_iter().collect();

        // Necessary : the states that are removed modify the id of all the states that come after
        v.sort();
        for j in (0..v.len()).rev() {
            self.del_state(&v[j])?;
        }
        Ok(())
    }
}

impl<'a, W: 'static + Semiring> MutableArcIterator<'a> for VectorFst<W> {
    type IterMut = slice::IterMut<'a, Arc<W>>;
    fn arcs_iter_mut(&'a mut self, state_id: &StateId) -> Result<Self::IterMut> {
        let state = self
            .states
            .get_mut(*state_id)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        Ok(state.arcs.iter_mut())
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
