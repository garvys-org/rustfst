use std::cmp::Ordering;

use anyhow::Result;

use crate::algorithms::tr_unique::tr_compare;
use crate::fst_impls::vector_fst::{VectorFst, VectorFstState};
use crate::fst_traits::MutableFst;
use crate::fst_traits::{CoreFst, MutableTrIterator};
use crate::semirings::Semiring;
use crate::{StateId, Tr};

#[inline]
fn equal_tr<W: Semiring>(tr_1: &Tr<W>, tr_2: &Tr<W>) -> bool {
    tr_1.ilabel == tr_2.ilabel && tr_1.olabel == tr_2.olabel && tr_1.nextstate == tr_2.nextstate
}

impl<W: 'static + Semiring> MutableFst for VectorFst<W> {
    fn new() -> Self {
        VectorFst {
            states: vec![],
            start_state: None,
            isymt: None,
            osymt: None,
        }
    }

    fn set_start(&mut self, state_id: StateId) -> Result<()> {
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

    fn set_final<S: Into<W>>(&mut self, state_id: StateId, final_weight: S) -> Result<()> {
        if let Some(state) = self.states.get_mut(state_id) {
            state.final_weight = Some(final_weight.into());
            Ok(())
        } else {
            bail!("Stateid {:?} doesn't exist", state_id);
        }
    }

    unsafe fn set_final_unchecked<S: Into<W>>(&mut self, state_id: usize, final_weight: S) {
        self.states.get_unchecked_mut(state_id).final_weight = Some(final_weight.into());
    }

    fn add_state(&mut self) -> StateId {
        let id = self.states.len();
        self.states.insert(id, VectorFstState::new());
        id
    }

    fn add_states(&mut self, n: usize) {
        let len = self.states.len();
        self.states.resize_with(len + n, VectorFstState::new);
    }

    fn del_state(&mut self, state_to_remove: StateId) -> Result<()> {
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

    fn del_states<T: IntoIterator<Item = StateId>>(&mut self, dstates: T) -> Result<()> {
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
            for (idx, tr) in unsafe { self.tr_iter_unchecked_mut(s).enumerate() } {
                let t = new_id[tr.nextstate];
                if t != -1 {
                    tr.nextstate = t as usize;
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

    unsafe fn del_trs_id_sorted_unchecked(&mut self, state: usize, to_del: &Vec<usize>) {
        let arcs = &mut self.states.get_unchecked_mut(state).arcs;
        for i in to_del.iter().rev() {
            arcs.remove(*i);
        }
    }

    fn add_tr(&mut self, source: StateId, tr: Tr<<Self as CoreFst>::W>) -> Result<()> {
        self.states
            .get_mut(source)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", source))?
            .arcs
            .push(tr);
        Ok(())
    }

    unsafe fn add_tr_unchecked(&mut self, source: usize, tr: Tr<Self::W>) {
        self.states.get_unchecked_mut(source).arcs.push(tr)
    }

    unsafe fn set_trs_unchecked(&mut self, source: usize, arcs: Vec<Tr<Self::W>>) {
        self.states.get_unchecked_mut(source).arcs = arcs
    }

    fn delete_final_weight(&mut self, source: usize) -> Result<()> {
        self.states
            .get_mut(source)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", source))?
            .final_weight = None;
        Ok(())
    }

    unsafe fn delete_final_weight_unchecked(&mut self, source: usize) {
        self.states.get_unchecked_mut(source).final_weight = None;
    }

    fn delete_trs(&mut self, source: usize) -> Result<()> {
        self.states
            .get_mut(source)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", source))?
            .arcs
            .clear();
        Ok(())
    }

    fn pop_trs(&mut self, source: usize) -> Result<Vec<Tr<Self::W>>> {
        let v = self
            .states
            .get_mut(source)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", source))?
            .arcs
            .drain(..)
            .collect();
        Ok(v)
    }

    unsafe fn pop_trs_unchecked(&mut self, source: usize) -> Vec<Tr<Self::W>> {
        self.states
            .get_unchecked_mut(source)
            .arcs
            .drain(..)
            .collect()
    }

    fn final_weight_mut(&mut self, state_id: StateId) -> Result<Option<&mut W>> {
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

    fn take_final_weight(&mut self, state_id: usize) -> Result<Option<Self::W>> {
        let s = self
            .states
            .get_mut(state_id)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        Ok(s.final_weight.take())
    }

    unsafe fn take_final_weight_unchecked(&mut self, state_id: usize) -> Option<Self::W> {
        self.states.get_unchecked_mut(state_id).final_weight.take()
    }

    fn sort_trs_unchecked<F: Fn(&Tr<Self::W>, &Tr<Self::W>) -> Ordering>(
        &mut self,
        state: StateId,
        f: F,
    ) {
        unsafe { self.states.get_unchecked_mut(state).arcs.sort_by(f) }
    }

    unsafe fn unique_trs_unchecked(&mut self, state: usize) {
        let arcs = &mut self.states.get_unchecked_mut(state).arcs;
        arcs.sort_by(tr_compare);
        arcs.dedup();
    }

    unsafe fn sum_trs_unchecked(&mut self, state: usize) {
        let arcs = &mut self.states.get_unchecked_mut(state).arcs;
        arcs.sort_by(tr_compare);
        let mut n_trs: usize = 0;
        for i in 0..arcs.len() {
            if n_trs > 0 && equal_tr(&arcs[i], &arcs[n_trs - 1]) {
                let (left, right) = arcs.split_at_mut(i);
                left[n_trs - 1]
                    .weight
                    .plus_assign(&right[0].weight)
                    .unwrap();
            } else {
                arcs.swap(n_trs, i);
                n_trs += 1;
            }
        }
        arcs.truncate(n_trs);
        // Truncate doesn't modify the capacity of the vector. Maybe a shrink_to_fit ?
    }
}

//#[test]
//fn lol() {
//    let mut fst = VectorFst::<crate::semirings::TropicalWeight>::new();
//    fst.set_final(0, 1.0).unwrap();
//}
