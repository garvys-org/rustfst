use std::cmp::Ordering;
use std::slice;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::tr_unique::tr_compare;
use crate::fst_impls::vector_fst::{VectorFst, VectorFstState};
use crate::fst_properties::mutable_properties::{
    add_state_properties, delete_all_states_properties, delete_states_properties,
    delete_trs_properties, set_final_properties, set_start_properties,
};
use crate::fst_properties::FstProperties;
use crate::fst_traits::CoreFst;
use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::trs_iter_mut::TrsIterMut;
use crate::{StateId, Tr};

#[inline]
fn equal_tr<W: Semiring>(tr_1: &Tr<W>, tr_2: &Tr<W>) -> bool {
    tr_1.ilabel == tr_2.ilabel && tr_1.olabel == tr_2.olabel && tr_1.nextstate == tr_2.nextstate
}

impl<W: Semiring> MutableFst<W> for VectorFst<W> {
    fn new() -> Self {
        VectorFst {
            states: vec![],
            start_state: None,
            isymt: None,
            osymt: None,
            properties: FstProperties::null_properties(),
        }
    }

    fn set_start(&mut self, state_id: StateId) -> Result<()> {
        ensure!(
            self.states.get(state_id).is_some(),
            "The state {:?} doesn't exist",
            state_id
        );
        self.start_state = Some(state_id);
        self.properties = set_start_properties(self.properties);
        Ok(())
    }

    unsafe fn set_start_unchecked(&mut self, state_id: usize) {
        self.start_state = Some(state_id);
        self.properties = set_start_properties(self.properties);
    }

    fn set_final<S: Into<W>>(&mut self, state_id: StateId, final_weight: S) -> Result<()> {
        if let Some(state) = self.states.get_mut(state_id) {
            let new_final_weight = final_weight.into();
            self.properties = set_final_properties(
                self.properties,
                state.final_weight.as_ref(),
                Some(&new_final_weight),
            );
            state.final_weight = Some(new_final_weight);
            Ok(())
        } else {
            bail!("Stateid {:?} doesn't exist", state_id);
        }
    }

    unsafe fn set_final_unchecked<S: Into<W>>(&mut self, state_id: usize, final_weight: S) {
        let new_final_weight = final_weight.into();
        self.properties = set_final_properties(
            self.properties,
            self.states
                .get_unchecked_mut(state_id)
                .final_weight
                .as_ref(),
            Some(&new_final_weight),
        );
        self.states.get_unchecked_mut(state_id).final_weight = Some(new_final_weight);
    }

    fn add_state(&mut self) -> StateId {
        let id = self.states.len();
        self.states.insert(id, VectorFstState::new());
        self.properties = add_state_properties(self.properties);
        id
    }

    fn add_states(&mut self, n: usize) {
        let len = self.states.len();
        self.states.resize_with(len + n, VectorFstState::new);
        self.properties = add_state_properties(self.properties);
    }

    fn tr_iter_mut(&mut self, state_id: StateId) -> Result<slice::IterMut<Tr<W>>> {
        let state = self
            .states
            .get_mut(state_id)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        let trs = Arc::make_mut(&mut state.trs.0);
        todo!("props");
        Ok(trs.iter_mut())
    }

    #[inline]
    unsafe fn tr_iter_unchecked_mut(&mut self, state_id: usize) -> slice::IterMut<Tr<W>> {
        let state = self.states.get_unchecked_mut(state_id);
        let trs = Arc::make_mut(&mut state.trs.0);
        todo!("props");
        trs.iter_mut()
    }

    fn del_state(&mut self, state_to_remove: StateId) -> Result<()> {
        // Remove the state from the vector
        // Check the trs for trs going to this state

        ensure!(
            state_to_remove < self.states.len(),
            "State id {:?} doesn't exist",
            state_to_remove
        );
        self.properties = delete_states_properties(self.properties);
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
            let trs_mut = Arc::make_mut(&mut self.states[s].trs.0);
            for (idx, tr) in trs_mut.iter_mut().enumerate() {
                let t = new_id[tr.nextstate];
                if t != -1 {
                    tr.nextstate = t as usize
                } else {
                    to_delete.push(idx);
                }
            }

            for i in to_delete.iter().rev() {
                self.states[s].trs.remove(*i);
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

        self.properties = delete_states_properties(self.properties);

        Ok(())
    }

    fn del_all_states(&mut self) {
        // Ensure the start state is no longer affected to a destroyed state
        self.start_state = None;

        // Remove all the states and thus the trs
        self.states.clear();

        self.properties = delete_all_states_properties();
    }

    unsafe fn del_trs_id_sorted_unchecked(&mut self, state: usize, to_del: &Vec<usize>) {
        let trs = &mut self.states.get_unchecked_mut(state).trs;
        for i in to_del.iter().rev() {
            trs.remove(*i);
        }
        todo!("props")
    }

    fn add_tr(&mut self, source: StateId, tr: Tr<W>) -> Result<()> {
        self.states
            .get_mut(source)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", source))?
            .trs
            .push(tr);
        self.update_properties_after_add_tr(source);
        Ok(())
    }

    unsafe fn add_tr_unchecked(&mut self, source: usize, tr: Tr<W>) {
        self.states.get_unchecked_mut(source).trs.push(tr);
        self.update_properties_after_add_tr(source);
    }

    unsafe fn set_trs_unchecked(&mut self, source: usize, trs: Vec<Tr<W>>) {
        let trs_inside = &mut self.states.get_unchecked_mut(source).trs;
        *Arc::make_mut(&mut trs_inside.0) = trs;
        todo!("props");
    }

    fn delete_final_weight(&mut self, source: usize) -> Result<()> {
        if let Some(s) = self.states.get_mut(source) {
            self.properties = set_final_properties(self.properties, s.final_weight.as_ref(), None);
            s.final_weight = None;
        } else {
            bail!("State {:?} doesn't exist", source)
        }
        Ok(())
    }

    unsafe fn delete_final_weight_unchecked(&mut self, source: usize) {
        let s = self.states.get_unchecked_mut(source);
        self.properties = set_final_properties(self.properties, s.final_weight.as_ref(), None);
        s.final_weight = None;
    }

    fn delete_trs(&mut self, source: usize) -> Result<()> {
        self.states
            .get_mut(source)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", source))?
            .trs
            .clear();
        self.properties = delete_trs_properties(self.properties);
        Ok(())
    }

    fn pop_trs(&mut self, source: usize) -> Result<Vec<Tr<W>>> {
        let trs = &mut self
            .states
            .get_mut(source)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", source))?
            .trs;
        let v = Arc::make_mut(&mut trs.0).drain(..).collect();
        self.properties = delete_trs_properties(self.properties);
        Ok(v)
    }

    unsafe fn pop_trs_unchecked(&mut self, source: usize) -> Vec<Tr<W>> {
        self.properties = delete_trs_properties(self.properties);
        let trs = &mut self.states.get_unchecked_mut(source).trs;
        Arc::make_mut(&mut trs.0).drain(..).collect()
    }

    fn take_final_weight(&mut self, state_id: usize) -> Result<Option<W>> {
        let s = self
            .states
            .get_mut(state_id)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;

        self.properties = set_final_properties(self.properties, s.final_weight.as_ref(), None);
        Ok(s.final_weight.take())
    }

    unsafe fn take_final_weight_unchecked(&mut self, state_id: usize) -> Option<W> {
        let s = self.states.get_unchecked_mut(state_id);
        self.properties = set_final_properties(self.properties, s.final_weight.as_ref(), None);
        s.final_weight.take()
    }

    fn sort_trs_unchecked<F: Fn(&Tr<W>, &Tr<W>) -> Ordering>(&mut self, state: StateId, f: F) {
        unsafe {
            let trs = &mut self.states.get_unchecked_mut(state).trs;
            Arc::make_mut(&mut trs.0).sort_by(f);
            todo!("props")
        }
    }

    unsafe fn unique_trs_unchecked(&mut self, state: usize) {
        let trs = &mut self.states.get_unchecked_mut(state).trs;
        let trs_vec = Arc::make_mut(&mut trs.0);
        trs_vec.sort_by(tr_compare);
        trs_vec.dedup();
        todo!("props")
    }

    unsafe fn sum_trs_unchecked(&mut self, state: usize) {
        let trs = &mut self.states.get_unchecked_mut(state).trs;
        let trs_vec = Arc::make_mut(&mut trs.0);
        trs_vec.sort_by(tr_compare);
        let mut n_trs: usize = 0;
        for i in 0..trs_vec.len() {
            if n_trs > 0 && equal_tr(&trs_vec[i], &trs_vec[n_trs - 1]) {
                let (left, right) = trs_vec.split_at_mut(i);
                left[n_trs - 1]
                    .weight
                    .plus_assign(&right[0].weight)
                    .unwrap();
            } else {
                trs_vec.swap(n_trs, i);
                n_trs += 1;
            }
        }
        trs_vec.truncate(n_trs);
        // Truncate doesn't modify the capacity of the vector. Maybe a shrink_to_fit ?
        todo!("props")
    }

    fn tr_iter_mut_revamp(&mut self, state_id: StateId) -> Result<TrsIterMut<W>> {
        let state = self
            .states
            .get_mut(state_id)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", state_id))?;
        let trs = Arc::make_mut(&mut state.trs.0);
        Ok(TrsIterMut::new(trs, &mut self.properties))
    }

    unsafe fn tr_iter_unchecked_mut_revamp(&mut self, state_id: StateId) -> TrsIterMut<W> {
        let state = self.states.get_unchecked_mut(state_id);
        let trs = Arc::make_mut(&mut state.trs.0);
        TrsIterMut::new(trs, &mut self.properties)
    }

    fn set_properties(&mut self, props: FstProperties) {
        self.properties |= props;
    }

    fn set_properties_with_mask(&mut self, props: FstProperties, mask: FstProperties) {
        self.properties &= !mask;
        self.properties |= props & mask;
    }
}
