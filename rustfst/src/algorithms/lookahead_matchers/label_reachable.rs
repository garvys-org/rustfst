use crate::algorithms::lookahead_matchers::interval_set::IntervalSet;
use crate::algorithms::lookahead_matchers::state_reachable::StateReachable;
use crate::fst_impls::VectorFst;
use crate::fst_traits::{CoreFst, ExpandedFst, MutableArcIterator, MutableFst};
use crate::semirings::Semiring;
use crate::{Arc, Label, StateId, EPS_LABEL, NO_LABEL};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use failure::Fallible;

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
}

pub struct LabelReachable<W: Semiring> {
    fst: VectorFst<W>,
    data: LabelReachableData,
    label2state: HashMap<Label, StateId>,
}

impl<W: Semiring + 'static> LabelReachable<W> {
    pub fn new(fst: VectorFst<W>, reach_input: bool) -> Fallible<Self> {
        // TODO: In OpenFst, the Fst is converted to a VectorFst
        let mut label_reachable = Self {
            fst,
            data: LabelReachableData::new(reach_input),
            label2state: HashMap::new(),
        };

        let nstates = label_reachable.fst.num_states();
        label_reachable.transform_fst();
        label_reachable.find_intervals(nstates)?;

        Ok(label_reachable)
    }

    // Redirects labeled arcs (input or output labels determined by ReachInput())
    // to new label-specific final states. Each original final state is
    // redirected via a transition labeled with kNoLabel to a new
    // kNoLabel-specific final state. Creates super-initial state for all states
    // with zero in-degree.
    fn transform_fst(&mut self) {
        let ins = self.fst.num_states();
        let mut ons = ins;
        let mut indeg = vec![0; ins];
        // Redirects labeled arcs to new final states.
        for s in 0..ins {
            for arc in unsafe { self.fst.arcs_iter_unchecked_mut(s) } {
                let label = if self.data.reach_input {
                    arc.ilabel
                } else {
                    arc.olabel
                };
                if label != EPS_LABEL {
                    arc.nextstate = match self.label2state.entry(label) {
                        Entry::Vacant(e) => {
                            let v = *e.insert(ons);
                            indeg.push(0);
                            ons += 1;
                            v
                        }
                        Entry::Occupied(e) => *e.get(),
                    };
                }
                indeg[arc.nextstate] += 1;
            }

            if let Some(final_weight) = unsafe { self.fst.final_weight_unchecked(s) } {
                if !final_weight.is_zero() {
                    let nextstate = match self.label2state.entry(NO_LABEL) {
                        Entry::Vacant(e) => {
                            let v = *e.insert(ons);
                            indeg.push(0);
                            ons += 1;
                            v
                        }
                        Entry::Occupied(e) => *e.get(),
                    };
                    unsafe {
                        self.fst.add_arc_unchecked(
                            s,
                            Arc::new(NO_LABEL, NO_LABEL, final_weight.clone(), nextstate),
                        )
                    };
                    indeg[nextstate] += 1;
                    unsafe { self.fst.delete_final_weight_unchecked(s) }
                }
            }
        }

        // Adds new final states to the FST.
        while self.fst.num_states() < ons {
            let s = self.fst.add_state();
            unsafe { self.fst.set_final_unchecked(s, W::one()) };
        }

        // Creates a super-initial state for all states with zero in-degree.
        let start = self.fst.add_state();
        unsafe { self.fst.set_start_unchecked(start) };
        for s in 0..start {
            if indeg[s] == 0 {
                unsafe {
                    self.fst
                        .add_arc_unchecked(start, Arc::new(0, 0, W::one(), s))
                };
            }
        }
    }

    fn find_intervals(&mut self, ins: StateId) -> Fallible<()> {
        let state_reachable = StateReachable::new(&self.fst)?;
        let state2index = &state_reachable.state2index;
        let interval_sets = &mut self.data.interval_sets;
        *interval_sets = state_reachable.isets;
        interval_sets.resize_with(ins, IntervalSet::default);
        let label2index = &mut self.data.label2index;
        for (label, state) in self.label2state.iter() {
            let i = state2index[*state];
            if *label == NO_LABEL {
                self.data.final_label = i;
            }
        }
        self.label2state.clear();
        Ok(())
    }
}
