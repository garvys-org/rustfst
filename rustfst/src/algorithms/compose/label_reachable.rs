use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::compose::{IntervalSet, StateReachable};
use crate::algorithms::tr_compares::{ILabelCompare, OLabelCompare};
use crate::algorithms::{fst_convert_from_ref, tr_sort};
use crate::fst_impls::VectorFst;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{CoreFst, ExpandedFst, Fst, MutableFst};
use crate::semirings::Semiring;
use crate::{Label, StateId, Tr, Trs, EPS_LABEL, NO_LABEL, UNASSIGNED};

#[derive(Debug, Clone, PartialEq)]
pub struct LabelReachableData {
    reach_input: bool,
    final_label: Label,
    label2index: HashMap<Label, Label>,
    interval_sets: Vec<IntervalSet>,
}

impl LabelReachableData {
    pub fn new(reach_input: bool) -> Self {
        Self {
            reach_input,
            final_label: NO_LABEL,
            label2index: HashMap::new(),
            interval_sets: Vec::new(),
        }
    }

    pub fn interval_set(&self, s: StateId) -> Result<&IntervalSet> {
        self.interval_sets
            .get(s as usize)
            .ok_or_else(|| format_err!("Missing state {}", s))
    }

    pub fn final_label(&self) -> Label {
        self.final_label
    }

    pub fn label2index(&self) -> &HashMap<Label, Label> {
        &self.label2index
    }

    pub fn reach_input(&self) -> bool {
        self.reach_input
    }

    pub fn relabel(&mut self, label: Label) -> Label {
        if label == EPS_LABEL {
            return EPS_LABEL;
        }
        let n = self.label2index.len();
        *self
            .label2index
            .entry(label)
            .or_insert_with(|| n as Label + 1)
    }

    pub fn relabel_fst<W: Semiring, F: MutableFst<W>>(
        &mut self,
        fst: &mut F,
        relabel_input: bool,
    ) -> Result<()> {
        for s in 0..(fst.num_states() as StateId) {
            unsafe {
                let mut it_tr = fst.tr_iter_unchecked_mut(s);
                for idx_tr in 0..it_tr.len() {
                    let tr = it_tr.get_unchecked(idx_tr);
                    if relabel_input {
                        let new_ilabel = self.relabel(tr.ilabel);
                        it_tr.set_ilabel_unchecked(idx_tr, new_ilabel);
                    } else {
                        let new_olabel = self.relabel(tr.olabel);
                        it_tr.set_olabel_unchecked(idx_tr, new_olabel);
                    }
                }
            }
        }

        if relabel_input {
            tr_sort(fst, ILabelCompare {});
            fst.take_input_symbols();
        } else {
            tr_sort(fst, OLabelCompare {});
            fst.take_output_symbols();
        }

        Ok(())
    }

    // Returns relabeling pairs (cf. relabel.h::Relabel()). If avoid_collisions is
    // true, extra pairs are added to ensure no collisions when relabeling
    // automata that have labels unseen here.
    pub fn relabel_pairs(&self, avoid_collisions: bool) -> Vec<(Label, Label)> {
        let mut pairs = vec![];
        for (key, val) in self.label2index.iter() {
            if *val != self.final_label {
                pairs.push((*key, *val));
            }
        }

        if avoid_collisions {
            for i in 1..=(self.label2index.len() as Label) {
                let it = self.label2index.get(&i);
                if it.is_none() || it.unwrap() == &self.final_label {
                    pairs.push((i, self.label2index.len() as Label + 1));
                }
            }
        }

        pairs
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelReachable {
    data: Arc<LabelReachableData>,
    reach_fst_input: bool,
}

impl LabelReachable {
    pub fn new<W: Semiring, F: Fst<W>>(fst: &F, reach_input: bool) -> Result<Self> {
        let data = Self::compute_data(fst, reach_input)?;

        Ok(Self {
            data: Arc::new(data),
            reach_fst_input: false,
        })
    }

    pub fn compute_data<W: Semiring, F: Fst<W>>(
        fst: &F,
        reach_input: bool,
    ) -> Result<LabelReachableData> {
        let mut fst: VectorFst<_> = fst_convert_from_ref(fst);

        let mut data = LabelReachableData::new(reach_input);
        let mut label2state = HashMap::new();

        let nstates = fst.num_states();
        Self::transform_fst(&mut fst, &mut data, &mut label2state);
        fst.compute_and_update_properties(FstProperties::ACYCLIC)?;
        Self::find_intervals(&fst, nstates as StateId, &mut data, &mut label2state)?;

        Ok(data)
    }

    pub fn new_from_data(data: Arc<LabelReachableData>) -> Self {
        Self {
            data,
            reach_fst_input: false,
        }
    }

    pub fn data(&self) -> &Arc<LabelReachableData> {
        &self.data
    }

    pub fn reach_input(&self) -> bool {
        self.data.reach_input
    }

    // Redirects labeled trs (input or output labels determined by ReachInput())
    // to new label-specific final states. Each original final state is
    // redirected via a transition labeled with kNoLabel to a new
    // kNoLabel-specific final state. Creates super-initial state for all states
    // with zero in-degree.
    fn transform_fst<W: Semiring>(
        fst: &mut VectorFst<W>,
        data: &mut LabelReachableData,
        label2state: &mut HashMap<Label, StateId>,
    ) {
        let ins = fst.num_states();
        let mut indeg = vec![0; ins];

        let ins = ins as StateId;
        let mut ons = ins;
        // Redirects labeled trs to new final states.
        for s in 0..ins {
            let mut it_tr = unsafe { fst.tr_iter_unchecked_mut(s) };
            for idx_tr in 0..it_tr.len() {
                let tr = unsafe { it_tr.get_unchecked(idx_tr) };

                let label = if data.reach_input {
                    tr.ilabel
                } else {
                    tr.olabel
                };

                let nextstate = if label != EPS_LABEL {
                    match label2state.entry(label) {
                        Entry::Vacant(e) => {
                            let v = *e.insert(ons);
                            indeg.push(0);
                            ons += 1;
                            v
                        }
                        Entry::Occupied(e) => *e.get(),
                    }
                } else {
                    tr.nextstate
                };
                indeg[nextstate as usize] += 1;
                unsafe { it_tr.set_nextstate_unchecked(idx_tr, nextstate) };
            }

            if let Some(final_weight) = unsafe { fst.final_weight_unchecked(s) } {
                if !final_weight.is_zero() {
                    let nextstate = match label2state.entry(NO_LABEL) {
                        Entry::Vacant(e) => {
                            let v = *e.insert(ons);
                            indeg.push(0);
                            ons += 1;
                            v
                        }
                        Entry::Occupied(e) => *e.get(),
                    };
                    unsafe {
                        fst.add_tr_unchecked(
                            s,
                            Tr::new(NO_LABEL, NO_LABEL, final_weight, nextstate),
                        )
                    };
                    indeg[nextstate as usize] += 1;
                    unsafe { fst.delete_final_weight_unchecked(s) }
                }
            }
        }

        // Adds new final states to the FST.
        while fst.num_states() < (ons as usize) {
            let s = fst.add_state();
            unsafe { fst.set_final_unchecked(s, W::one()) };
        }

        // Creates a super-initial state for all states with zero in-degree.
        let start = fst.add_state();
        unsafe { fst.set_start_unchecked(start) };
        for s in 0..start {
            if indeg[s as usize] == 0 {
                unsafe { fst.add_tr_unchecked(start, Tr::new(0, 0, W::one(), s)) };
            }
        }
    }

    fn find_intervals<W: Semiring>(
        fst: &VectorFst<W>,
        ins: StateId,
        data: &mut LabelReachableData,
        label2state: &mut HashMap<Label, StateId>,
    ) -> Result<()> {
        let state_reachable = StateReachable::new(fst)?;
        let state2index = &state_reachable.state2index;
        let interval_sets = &mut data.interval_sets;
        *interval_sets = state_reachable.isets;
        interval_sets.resize_with(ins as usize, IntervalSet::default);

        let label2index = &mut data.label2index;

        for (label, state) in label2state.iter() {
            let i = state2index[*state as usize];
            label2index.insert(*label, i as Label);
            if *label == NO_LABEL {
                data.final_label = i as Label;
            }
        }
        label2state.clear();
        Ok(())
    }

    pub fn reach_init<W: Semiring, F: Fst<W>>(&mut self, fst: &F, reach_input: bool) -> Result<()> {
        self.reach_fst_input = reach_input;

        let true_prop = if self.reach_fst_input {
            FstProperties::I_LABEL_SORTED
        } else {
            FstProperties::O_LABEL_SORTED
        };

        let props = fst.properties_check(true_prop)?;

        if !props.contains(true_prop) {
            bail!("LabelReachable::ReachInit: Fst is not sorted")
        }
        Ok(())
    }

    // Can reach this label from current state?
    // Original labels must be transformed by the Relabel methods above.
    pub fn reach_label(&self, current_state: StateId, label: Label) -> Result<bool> {
        if label == EPS_LABEL {
            return Ok(false);
        }
        Ok(self
            .data
            .interval_set(current_state)?
            .member(label as usize))
    }

    // Can reach final state (via epsilon transitions) from this state?
    pub fn reach_final(&self, current_state: StateId) -> Result<bool> {
        Ok(self
            .data
            .interval_set(current_state)?
            .member(self.data.final_label() as usize))
    }

    pub fn reach<'a, W: Semiring + 'a, T: Trs<W>>(
        &self,
        current_state: StateId,
        trs: T,
        aiter_begin: usize,
        aiter_end: usize,
        compute_weight: bool,
    ) -> Result<Option<(usize, usize, W)>> {
        let mut reach_begin = UNASSIGNED;
        let mut reach_end = UNASSIGNED;
        let mut reach_weight = W::zero();
        let interval_set = self.data.interval_set(current_state)?;
        let trs_slice = trs.trs();
        if 2 * (aiter_end - aiter_begin) < interval_set.len() {
            let mut reach_label = NO_LABEL;
            for pos in aiter_begin..aiter_end {
                let tr = unsafe { trs_slice.get_unchecked(pos) };
                let label = if self.reach_fst_input {
                    tr.ilabel
                } else {
                    tr.olabel
                };
                if label == reach_label || self.reach_label(current_state, label)? {
                    reach_label = label;
                    if reach_begin == UNASSIGNED {
                        reach_begin = pos;
                    }
                    reach_end = pos + 1;
                    if compute_weight {
                        reach_weight.plus_assign(&tr.weight)?;
                    }
                }
            }
        } else {
            let mut begin_low;
            let mut end_low = aiter_begin;
            for interval in interval_set.iter() {
                begin_low =
                    self.lower_bound(trs_slice, end_low, aiter_end, interval.begin as StateId);
                end_low =
                    self.lower_bound(trs_slice, begin_low, aiter_end, interval.end as StateId);
                if end_low - begin_low > 0 {
                    if reach_begin == UNASSIGNED {
                        reach_begin = begin_low;
                    }
                    reach_end = end_low;
                    if compute_weight {
                        for i in begin_low..end_low {
                            reach_weight
                                .plus_assign(unsafe { &trs_slice.get_unchecked(i).weight })?;
                        }
                    }
                }
            }
        }

        if reach_begin != UNASSIGNED {
            Ok(Some((reach_begin, reach_end, reach_weight)))
        } else {
            Ok(None)
        }
    }

    fn lower_bound<W: Semiring>(
        &self,
        trs: &[Tr<W>],
        aiter_begin: usize,
        aiter_end: usize,
        match_label: Label,
    ) -> usize {
        debug_assert!(match_label != NO_LABEL);
        let mut low = aiter_begin;
        let mut high = aiter_end;
        while low < high {
            let mid = low + (high - low) / 2;
            let tr = unsafe { trs.get_unchecked(mid) };
            let label = if self.reach_fst_input {
                tr.ilabel
            } else {
                tr.olabel
            };
            debug_assert!(label != NO_LABEL);
            if label < match_label {
                low = mid + 1;
            } else {
                high = mid;
            }
        }

        low
    }
}
