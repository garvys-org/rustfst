use std::cmp::Ordering;

use failure::Fallible;

use crate::algorithms::arc_unique::arc_compare;
use crate::fst_impls::vector_fst::{VectorFst, VectorFstState};
use crate::fst_traits::MutableFst;
use crate::fst_traits::{CoreFst, MutableArcIterator};
use crate::semirings::Semiring;
use crate::{Arc, StateId, SymbolTable};
use std::rc::Rc;

#[inline]
fn equal_arc<W: Semiring>(arc_1: &Arc<W>, arc_2: &Arc<W>) -> bool {
    arc_1.ilabel == arc_2.ilabel
        && arc_1.olabel == arc_2.olabel
        && arc_1.nextstate == arc_2.nextstate
}

impl<W: 'static + Semiring> MutableFst for VectorFst<W> {
    fn new() -> Self {
        VectorFst {
            states: vec![],
            start_state: None,
            isymt: None,
            osymt: None
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

    unsafe fn set_start_unchecked(&mut self, state_id: usize) {
        self.start_state = Some(state_id);
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
            for (idx, arc) in unsafe { self.arcs_iter_unchecked_mut(s).enumerate() } {
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

    fn del_all_states(&mut self) {
        // Ensure the start state is no longer affected to a destroyed state
        self.start_state = None;

        // Remove all the states and thus the arcs
        self.states.clear();
    }

    unsafe fn del_arcs_id_sorted_unchecked(&mut self, state: usize, to_del: &Vec<usize>) {
        let ref mut arcs = self.states.get_unchecked_mut(state).arcs;
        for i in to_del.iter().rev() {
            arcs.remove(*i);
        }
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

    fn final_weight_mut(&mut self, state_id: StateId) -> Fallible<Option<&mut W>> {
        let s = self
            .states
            .get_mut(state_id)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        Ok(s.final_weight.as_mut())
    }

    unsafe fn final_weight_unchecked_mut(&mut self, state_id: usize) -> Option<&mut Self::W> {
        self.states
            .get_unchecked_mut(state_id)
            .final_weight
            .as_mut()
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

    fn input_symbols(&self) -> Option<Rc<SymbolTable>> {
        // Rc is incremented, SymbolTable is not duplicated
        self.isymt.clone()
    }

    fn set_input_symbols(&mut self, symt: Rc<SymbolTable>) {
        self.isymt = Some(Rc::clone(&symt))
    }

    fn output_symbols(&self) -> Option<Rc<SymbolTable>> {
        self.osymt.clone()
    }

    fn set_output_symbols(&mut self, symt: Rc<SymbolTable>) {
        self.osymt = Some(Rc::clone(&symt));
    }
}
