use std::fmt;
use std::ops::{Add, BitOr};
use std::slice;

use failure::{bail, ensure, format_err, Fallible};

use crate::algorithms::arc_unique::arc_compare;
use crate::algorithms::{concat, union};
use crate::arc::Arc;
use crate::fst_traits::{
    ArcIterator, CoreFst, ExpandedFst, FinalStatesIterator, Fst, MutableArcIterator, MutableFst,
    StateIterator, TextParser,
};
use crate::parsers::text_fst::ParsedTextFst;
use crate::semirings::Semiring;
use crate::StateId;
use std::cmp::Ordering;

/// Simple concrete, mutable FST whose states and arcs are stored in standard vectors.
///
/// All states are stored in a vector of states.
/// In each state, there is a vector of arcs containing the outgoing transitions.
#[derive(Debug, PartialEq, Clone)]
pub struct VectorFst<W: Semiring> {
    pub(crate) states: Vec<VectorFstState<W>>,
    pub(crate) start_state: Option<StateId>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VectorFstState<W: Semiring> {
    pub(crate) final_weight: Option<W>,
    pub(crate) arcs: Vec<Arc<W>>,
}

impl<W: 'static + Semiring> Fst for VectorFst<W> {}

impl<W: 'static + Semiring> CoreFst for VectorFst<W> {
    type W = W;
    fn start(&self) -> Option<StateId> {
        self.start_state
    }

    fn final_weight(&self, state_id: StateId) -> Option<W> {
        if let Some(state) = self.states.get(state_id) {
            state.final_weight.clone()
        } else {
            None
        }
    }

    fn num_arcs(&self, s: StateId) -> Fallible<usize> {
        if let Some(vector_fst_state) = self.states.get(s) {
            Ok(vector_fst_state.num_arcs())
        } else {
            bail!("State {:?} doesn't exist", s);
        }
    }

    fn num_arcs_unchecked(&self, s: usize) -> usize {
        unsafe { self.states.get_unchecked(s).num_arcs() }
    }
}

impl<'a, W: 'a + Semiring> StateIterator<'a> for VectorFst<W> {
    type Iter = VectorStateIterator<'a, W>;
    fn states_iter(&'a self) -> Self::Iter {
        VectorStateIterator::new(self)
    }
}

#[derive(Clone)]
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
    fn arcs_iter(&'a self, state_id: StateId) -> Fallible<Self::Iter> {
        let state = self
            .states
            .get(state_id)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        Ok(state.arcs.iter())
    }

    unsafe fn arcs_iter_unchecked(&'a self, state_id: usize) -> Self::Iter {
        self.states.get_unchecked(state_id).arcs.iter()
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

    fn set_start(&mut self, state_id: StateId) -> Fallible<()> {
        ensure!(
            self.states.get(state_id).is_some(),
            "The state {:?} doesn't exist",
            state_id
        );
        self.start_state = Some(state_id);
        Ok(())
    }

    fn set_final(&mut self, state_id: StateId, final_weight: W) -> Fallible<()> {
        if let Some(state) = self.states.get_mut(state_id) {
            state.final_weight = Some(final_weight);
            Ok(())
        } else {
            bail!("Stateid {:?} doesn't exist", state_id);
        }
    }

    unsafe fn set_final_unchecked(&mut self, state_id: usize, final_weight: Self::W) {
        self.states.get_unchecked_mut(state_id).final_weight = Some(final_weight);
    }

    fn add_state(&mut self) -> StateId {
        let id = self.states.len();
        self.states.insert(id, VectorFstState::default());
        id
    }

    fn add_states(&mut self, n: usize) {
        let len = self.states.len();
        self.states.resize_with(len + n, VectorFstState::default);
    }

    fn del_state(&mut self, state_to_remove: StateId) -> Fallible<()> {
        // Remove the state from the vector
        // Check the arcs for arcs going to this state

        ensure!(
            state_to_remove < self.states.len(),
            "State id {:?} doesn't exist",
            state_to_remove
        );
        let v = vec![state_to_remove];
        self.del_states(v.into_iter())
    }

    fn del_states<T: IntoIterator<Item = StateId>>(&mut self, dstates: T) -> Fallible<()> {
        let mut new_id = vec![0 as i32; self.states.len()];

        for s in dstates {
            new_id[s] = -1;
        }

        let mut nstates = 0 as usize;

        for s in 0..self.states.len() {
            if new_id[s] != -1 {
                new_id[s] = nstates as i32;
                if s != nstates {
                    self.states.swap(nstates, s);
                }
                nstates += 1;
            }
        }

        self.states.truncate(nstates);

        for s in 0..self.states.len() {
            let mut to_delete = vec![];
            for (idx, arc) in unsafe{self.arcs_iter_unchecked_mut(s).enumerate()} {
                let t = new_id[arc.nextstate];
                if t != -1 {
                    arc.nextstate = t as usize;
                } else {
                    to_delete.push(idx);
                }
            }
            for i in to_delete.iter().rev() {
                self.states[s].arcs.remove(*i);
            }
        }

        if let Some(start) = self.start() {
            let new_state = new_id[start];
            if new_state == -1 {
                self.start_state = None;
            } else {
                self.start_state = Some(new_state as usize);
            }
        }

        Ok(())
    }

    fn add_arc(&mut self, source: StateId, arc: Arc<<Self as CoreFst>::W>) -> Fallible<()> {
        self.states
            .get_mut(source)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", source))?
            .arcs
            .push(arc);
        Ok(())
    }

    unsafe fn add_arc_unchecked(&mut self, source: usize, arc: Arc<Self::W>) {
        self.states.get_unchecked_mut(source).arcs.push(arc)
    }

    unsafe fn set_arcs_unchecked(&mut self, source: usize, arcs: Vec<Arc<Self::W>>) {
        self.states.get_unchecked_mut(source).arcs = arcs
    }

    fn delete_final_weight(&mut self, source: usize) -> Fallible<()> {
        self.states
            .get_mut(source)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", source))?
            .final_weight = None;
        Ok(())
    }

    fn delete_arcs(&mut self, source: usize) -> Fallible<()> {
        self.states
            .get_mut(source)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", source))?
            .arcs
            .clear();
        Ok(())
    }

    fn pop_arcs(&mut self, source: usize) -> Fallible<Vec<Arc<Self::W>>> {
        let v = self
            .states
            .get_mut(source)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", source))?
            .arcs
            .drain(..)
            .collect();
        Ok(v)
    }

    unsafe fn pop_arcs_unchecked(&mut self, source: usize) -> Vec<Arc<Self::W>> {
            self.states
                .get_unchecked_mut(source)
                .arcs
                .drain(..)
                .collect()
    }

    fn reserve_arcs(&mut self, source: usize, additional: usize) -> Fallible<()> {
        self.states
            .get_mut(source)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", source))?
            .arcs
            .reserve(additional);
        Ok(())
    }

    #[inline]
    unsafe fn reserve_arcs_unchecked(&mut self, source: usize, additional: usize) {
            self.states
                .get_unchecked_mut(source)
                .arcs
                .reserve(additional)
    }

    fn reserve_states(&mut self, additional: usize) {
        self.states.reserve(additional);
    }

    fn final_weight_mut(&mut self, state_id: StateId) -> Option<&mut W> {
        if let Some(state) = self.states.get_mut(state_id) {
            state.final_weight.as_mut()
        } else {
            None
        }
    }

    fn sort_arcs_unchecked<F: Fn(&Arc<Self::W>, &Arc<Self::W>) -> Ordering>(
        &mut self,
        state: StateId,
        f: F,
    ) {
        unsafe { self.states.get_unchecked_mut(state).arcs.sort_by(f) }
    }

    unsafe fn unique_arcs_unchecked(&mut self, state: usize) {
        let arcs = &mut self.states.get_unchecked_mut(state).arcs;
        arcs.sort_by(arc_compare);
        arcs.dedup();
    }

    unsafe fn sum_arcs_unchecked(&mut self, state: usize) {
        let arcs = &mut self.states.get_unchecked_mut(state).arcs;
        arcs.sort_by(arc_compare);
        let mut n_arcs: usize = 0;
        for i in 0..arcs.len() {
            if n_arcs > 0 && equal_arc(&arcs[i], &arcs[n_arcs - 1]) {
                let (left, right) = arcs.split_at_mut(i);
                left[n_arcs - 1]
                    .weight
                    .plus_assign(&right[0].weight)
                    .unwrap();
            } else {
                arcs.swap(n_arcs, i);
                n_arcs += 1;
            }
        }
        arcs.truncate(n_arcs);
        // Truncate doesn't modify the capacity of the vector. Maybe a shrink_to_fit ?
    }
}

#[inline]
fn equal_arc<W: Semiring>(arc_1: &Arc<W>, arc_2: &Arc<W>) -> bool {
    arc_1.ilabel == arc_2.ilabel
        && arc_1.olabel == arc_2.olabel
        && arc_1.nextstate == arc_2.nextstate
}

impl<'a, W: 'static + Semiring> MutableArcIterator<'a> for VectorFst<W> {
    type IterMut = slice::IterMut<'a, Arc<W>>;
    fn arcs_iter_mut(&'a mut self, state_id: StateId) -> Fallible<Self::IterMut> {
        let state = self
            .states
            .get_mut(state_id)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        Ok(state.arcs.iter_mut())
    }

    #[inline]
    unsafe fn arcs_iter_unchecked_mut(&'a mut self, state_id: usize) -> Self::IterMut {
        self.states.get_unchecked_mut(state_id).arcs.iter_mut()
    }
}

impl<W: Semiring> VectorFstState<W> {
    pub fn num_arcs(&self) -> usize {
        self.arcs.len()
    }
}

impl<W: 'static + Semiring<Type = f32>> TextParser for VectorFst<W> {
    fn from_parsed_fst_text(parsed_fst_text: ParsedTextFst) -> Fallible<Self> {
        let start_state = parsed_fst_text.start();
        let num_states = parsed_fst_text.num_states();

        let states = vec![VectorFstState::<W>::default(); num_states];

        let mut fst = VectorFst {
            states,
            start_state,
        };

        for transition in parsed_fst_text.transitions.into_iter() {
            let weight = transition.weight.map(W::new).unwrap_or_else(W::one);
            let arc = Arc::new(
                transition.ilabel,
                transition.olabel,
                weight,
                transition.nextstate,
            );
            fst.add_arc(transition.state, arc)?;
        }

        for final_state in parsed_fst_text.final_states.into_iter() {
            let weight = final_state.weight.map(W::new).unwrap_or_else(W::one);
            fst.set_final(final_state.state, weight)?;
        }

        Ok(fst)
    }
}

add_or_fst!(W, VectorFst<W>);
display_fst_trait!(W, VectorFst<W>);
