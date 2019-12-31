use std::collections::{BTreeSet, HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::slice::Iter as IterSlice;

use bimap::BiHashMap;
use bitflags::_core::cell::{Ref, RefCell};
use failure::{bail, Fallible};
use nom::lib::std::collections::VecDeque;

use crate::{Arc, EPS_LABEL, Label, StateId};
use crate::algorithms::cache::CacheImpl;
use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::semirings::Semiring;
use crate::algorithms::replace::ReplaceLabelType::{ReplaceLabelNeither, ReplaceLabelInput};

/// This specifies what labels to output on the call or return arc.
#[derive(PartialOrd, PartialEq, Copy, Clone)]
enum ReplaceLabelType {
    /// Epsilon labels on both input and output.
    ReplaceLabelNeither,
    /// Non-epsilon labels on input and epsilon on output.
    ReplaceLabelInput,
    /// Epsilon on input and non-epsilon on output.
    ReplaceLabelOutput,
    /// Non-epsilon labels on both input and output.
    ReplaceLabelBoth,
}

#[allow(unused)]
struct ReplaceFstOptions {
    root: Label,
    call_label_type: ReplaceLabelType,
    return_label_type: ReplaceLabelType,
    call_output_label: Option<Label>,
    return_label: Label,
}

impl ReplaceFstOptions {
    pub fn new(root: Label, epsilon_on_replace: bool) -> Self {
        Self {
            root,
            call_label_type: if epsilon_on_replace {ReplaceLabelNeither} else {ReplaceLabelInput},
            return_label_type: ReplaceLabelNeither,
            call_output_label: if epsilon_on_replace {Some(0)} else {None},
            return_label: 0
        }
    }
}

pub fn replace<F1, F2>(fst_list: Vec<(Label, F1)>, root: Label, epsilon_on_replace: bool) -> Fallible<F2>
where
    F1: ExpandedFst,
    F1::W: 'static,
    F2: MutableFst<W=F1::W> + ExpandedFst<W=F1::W>
{
    let opts = ReplaceFstOptions::new(root, epsilon_on_replace);
    let mut fst = ReplaceFstImpl::new(fst_list, opts)?;
    fst.compute()
}

#[allow(unused)]
/// Returns true if label type on arc results in epsilon input label.
fn epsilon_on_input(label_type: ReplaceLabelType) -> bool {
    label_type == ReplaceLabelType::ReplaceLabelNeither
        || label_type == ReplaceLabelType::ReplaceLabelOutput
}

#[allow(unused)]
/// Returns true if label type on arc results in epsilon input label.
fn epsilon_on_output(label_type: ReplaceLabelType) -> bool {
    label_type == ReplaceLabelType::ReplaceLabelNeither
        || label_type == ReplaceLabelType::ReplaceLabelInput
}

#[allow(unused)]
fn replace_transducer(
    call_label_type: ReplaceLabelType,
    return_label_type: ReplaceLabelType,
    call_output_label: Option<Label>,
) -> bool {
    call_label_type == ReplaceLabelType::ReplaceLabelInput
        || call_label_type == ReplaceLabelType::ReplaceLabelOutput
        || (call_label_type == ReplaceLabelType::ReplaceLabelBoth && call_output_label.is_some())
        || return_label_type == ReplaceLabelType::ReplaceLabelInput
        || return_label_type == ReplaceLabelType::ReplaceLabelOutput
}

#[allow(unused)]
struct ReplaceFstImpl<F: ExpandedFst> {
    cache_impl: CacheImpl<F::W>,
    call_label_type_: ReplaceLabelType,
    return_label_type_: ReplaceLabelType,
    call_output_label_: Option<Label>,
    return_label_: Label,
    fst_array: Vec<F>,
    nonterminal_set: BTreeSet<Label>,
    nonterminal_hash: HashMap<Label, Label>,
    root: Label,
    state_table: ReplaceStateTable,
}

impl<F: ExpandedFst> ReplaceFstImpl<F> {
    #[allow(unused)]
    fn new(fst_list: Vec<(Label, F)>, opts: ReplaceFstOptions) -> Fallible<Self> {
        let mut replace_fst_impl = Self {
            cache_impl: CacheImpl::new(),
            call_label_type_: opts.call_label_type,
            return_label_type_: opts.return_label_type,
            call_output_label_: opts.call_output_label,
            return_label_: opts.return_label,
            fst_array: Vec::with_capacity(fst_list.len()),
            nonterminal_set: BTreeSet::new(),
            nonterminal_hash: HashMap::new(),
            root: 0,
            state_table: ReplaceStateTable::new(),
        };

        if let Some(v) = replace_fst_impl.call_output_label_ {
            if v == EPS_LABEL {
                replace_fst_impl.call_label_type_ = ReplaceLabelType::ReplaceLabelNeither;
            }
        }

        if replace_fst_impl.return_label_ == 0 {
            replace_fst_impl.return_label_type_ = ReplaceLabelType::ReplaceLabelNeither;
        }

        for (idx, (label, fst)) in fst_list.into_iter().enumerate() {
            replace_fst_impl
                .nonterminal_hash
                .insert(label, replace_fst_impl.fst_array.len());
            replace_fst_impl.nonterminal_set.insert(label);
            replace_fst_impl.fst_array.push(fst);
        }

        match replace_fst_impl.nonterminal_hash.entry(opts.root) {
            Entry::Vacant(_) => bail!(
                "ReplaceFstImpl: No FST corresponding to root label {} in the input tuple vector",
                opts.root
            ),
            Entry::Occupied(e) => {
                replace_fst_impl.root = *e.get();
            }
        };

        Ok(replace_fst_impl)
    }

    pub fn arcs_iter(&mut self, state: StateId) -> Fallible<IterSlice<Arc<F::W>>> {
        if !self.cache_impl.expanded(state) {
            self.expand(state)?;
        }
        self.cache_impl.arcs_iter(state)
    }

    #[allow(unused)]
    fn start(&mut self) -> Fallible<Option<StateId>> {
        if !self.cache_impl.has_start() {
            let start = self.compute_start()?;
            self.cache_impl.set_start(start);
        }
        Ok(self.cache_impl.start().unwrap())
    }

    fn compute_start(&mut self) -> Fallible<Option<StateId>> {
        if self.fst_array.is_empty() {
            return Ok(None);
        } else {
            if let Some(fst_start) = self.fst_array[self.root].start() {
                let prefix = self.get_prefix_id(&ReplaceStackPrefix::new());
                let start = self
                    .state_table
                    .tuple_table
                    .find_id(&ReplaceStateTuple::new(
                        prefix,
                        Some(self.root),
                        Some(fst_start),
                    ));
                return Ok(Some(start));
            } else {
                return Ok(None);
            }
        }
    }

    #[allow(unused)]
    pub fn final_weight(&mut self, state: StateId) -> Fallible<Option<&F::W>> {
        if !self.cache_impl.has_final(state) {
            let final_weight = self.compute_final(state)?;
            self.cache_impl.set_final_weight(state, final_weight)?;
        }
        self.cache_impl.final_weight(state)
    }

    #[allow(unused)]
    fn compute_final(&mut self, state: StateId) -> Fallible<Option<F::W>> {
        let tuple = self.state_table.tuple_table.find_tuple(state);
        if tuple.prefix_id == 0 {
            let fst_state = tuple.fst_state;
            self.fst_array
                .get(tuple.fst_id.unwrap())
                .unwrap()
                .final_weight(tuple.fst_state.unwrap())
                .map(|e| e.cloned())
        } else {
            Ok(None)
        }
    }

    #[allow(unused)]
    fn expand(&mut self, state: StateId) -> Fallible<()> {
        //        let tuple = self.state_table.tuple_table.find_tuple(state);

        //        if tuple.fst_state.is_some() {
        if let Some(arc) = self.compute_final_arc(state) {
            self.cache_impl.push_arc(state, arc)?;
        }

        let tuple = self.state_table.tuple_table.find_tuple(state).clone();
        for arc in self
            .fst_array
            .get(tuple.fst_id.unwrap())
            .unwrap()
            .arcs_iter(tuple.fst_state.unwrap())?
        {
            if let Some(new_arc) = self.compute_arc(&tuple, arc) {
                self.cache_impl.push_arc(state, new_arc);
            }
        }

        self.cache_impl.mark_expanded(state);
        Ok(())
    }

    fn compute_final_arc(&mut self, state: StateId) -> Option<Arc<F::W>> {
        let tuple = self.state_table.tuple_table.find_tuple(state);
        let fst_state = tuple.fst_state;
        if fst_state.is_none() {
            return None;
        }
        if self
            .fst_array
            .get(tuple.fst_id.unwrap())
            .as_ref()
            .unwrap()
            .is_final(fst_state.unwrap())
            .unwrap()
            && tuple.prefix_id > 0
        {
            let tuple = tuple.clone();
            //            let mut arc = Arc::new();
            let ilabel = if epsilon_on_input(self.return_label_type_) {
                EPS_LABEL
            } else {
                self.return_label_
            };
            let olabel = if epsilon_on_output(self.return_label_type_) {
                0
            } else {
                self.return_label_
            };
            let stack = self
                .state_table
                .prefix_table
                .find_tuple(tuple.prefix_id)
                .clone();
            let top = stack.top();
            let prefix_id = self.pop_prefix(stack.clone());
            let nextstate = self
                .state_table
                .tuple_table
                .find_id(&ReplaceStateTuple::new(
                    prefix_id,
                    top.fst_id,
                    top.nextstate,
                ));
            if let Some(weight) = self
                .fst_array
                .get(tuple.fst_id.unwrap())
                .as_ref()
                .unwrap()
                .final_weight(fst_state.unwrap())
                .unwrap()
            {
                return Some(Arc::new(ilabel, olabel, weight.clone(), nextstate));
            }
            None
        } else {
            None
        }
    }

    fn get_prefix_id(&self, prefix: &ReplaceStackPrefix) -> StateId {
        self.state_table.prefix_table.find_id(prefix)
    }

    fn pop_prefix(&self, mut prefix: ReplaceStackPrefix) -> StateId {
        prefix.pop();
        self.get_prefix_id(&prefix)
    }

    fn push_prefix(
        &self,
        mut prefix: ReplaceStackPrefix,
        fst_id: Option<Label>,
        nextstate: Option<StateId>,
    ) -> StateId {
        prefix.push(fst_id, nextstate);
        self.get_prefix_id(&prefix)
    }

    fn compute_arc<W: Semiring>(&self, tuple: &ReplaceStateTuple, arc: &Arc<W>) -> Option<Arc<W>> {
        if !epsilon_on_input(self.call_label_type_) {
            return Some(arc.clone());
        }
        if arc.olabel == EPS_LABEL
            || arc.olabel < *self.nonterminal_set.iter().next().unwrap()
            || arc.olabel > *self.nonterminal_set.iter().rev().next().unwrap()
        {
            let state_tuple =
                ReplaceStateTuple::new(tuple.prefix_id, tuple.fst_id, Some(arc.nextstate));
            let nextstate = self.state_table.tuple_table.find_id(&state_tuple);
            return Some(Arc::new(
                arc.ilabel,
                arc.olabel,
                arc.weight.clone(),
                nextstate,
            ));
        } else {
            // Checks for non-terminal
            if let Some(nonterminal) = self.nonterminal_hash.get(&arc.olabel) {
                let p = self
                    .state_table
                    .prefix_table
                    .find_tuple(tuple.prefix_id)
                    .clone();
                let nt_prefix = self.push_prefix(p, tuple.fst_id, Some(arc.nextstate));
                if let Some(nt_start) = self.fst_array.get(*nonterminal).unwrap().start() {
                    let nt_nextstate =
                        self.state_table
                            .tuple_table
                            .find_id(&ReplaceStateTuple::new(
                                nt_prefix,
                                Some(*nonterminal),
                                Some(nt_start),
                            ));
                    let ilabel = if epsilon_on_input(self.call_label_type_) {
                        0
                    } else {
                        arc.ilabel
                    };
                    let olabel = if epsilon_on_output(self.call_label_type_) {
                        0
                    } else {
                        self.call_output_label_.unwrap_or(arc.olabel)
                    };
                    return Some(Arc::new(ilabel, olabel, arc.weight.clone(), nt_nextstate));
                } else {
                    return None;
                }
            } else {
                let nextstate = self
                    .state_table
                    .tuple_table
                    .find_id(&ReplaceStateTuple::new(
                        tuple.prefix_id,
                        tuple.fst_id,
                        Some(arc.nextstate),
                    ));
                return Some(Arc::new(
                    arc.ilabel,
                    arc.olabel,
                    arc.weight.clone(),
                    nextstate,
                ));
            }
        }
    }

    pub fn compute<F2: MutableFst<W = F::W> + ExpandedFst<W = F::W>>(&mut self) -> Fallible<F2>
        where
            F::W: 'static,
    {
        let start_state = self.start()?;
        let mut fst_out = F2::new();
        if start_state.is_none() {
            return Ok(fst_out);
        }
        let start_state = start_state.unwrap();
        for _ in 0..=start_state {
            fst_out.add_state();
        }
        fst_out.set_start(start_state)?;
        let mut queue = VecDeque::new();
        let mut visited_states = HashSet::new();
        visited_states.insert(start_state);
        queue.push_back(start_state);
        while !queue.is_empty() {
            let s = queue.pop_front().unwrap();
            for arc in self.arcs_iter(s)? {
                if !visited_states.contains(&arc.nextstate) {
                    queue.push_back(arc.nextstate);
                    visited_states.insert(arc.nextstate);
                }
                let n = fst_out.num_states();
                for _ in n..=arc.nextstate {
                    fst_out.add_state();
                }
                fst_out.add_arc(s, arc.clone())?;
            }
            if let Some(f_w) = self.final_weight(s)? {
                fst_out.set_final(s, f_w.clone())?;
            }
        }
        Ok(fst_out)
    }
}

#[derive(Hash, Eq, PartialOrd, PartialEq, Clone)]
struct PrefixTuple {
    fst_id: Option<Label>,
    nextstate: Option<StateId>,
}

impl PrefixTuple {
    fn new(fst_id: Option<Label>, nextstate: Option<StateId>) -> Self {
        Self { fst_id, nextstate }
    }
}

#[derive(Hash, Eq, PartialOrd, PartialEq, Clone)]
struct ReplaceStackPrefix {
    prefix: Vec<PrefixTuple>,
}

impl ReplaceStackPrefix {
    fn new() -> Self {
        Self { prefix: vec![] }
    }

    fn push(&mut self, fst_id: Option<StateId>, nextstate: Option<StateId>) {
        self.prefix.push(PrefixTuple { fst_id, nextstate });
    }

    fn pop(&mut self) {
        self.prefix.pop();
    }

    fn top(&self) -> &PrefixTuple {
        self.prefix.last().as_ref().unwrap()
    }

    fn depth(&self) -> usize {
        self.prefix.len()
    }
}

#[derive(Hash, Eq, PartialOrd, PartialEq, Clone)]
struct ReplaceStateTuple {
    /// Index in prefix table.
    prefix_id: usize,
    /// Current FST being walked.
    fst_id: Option<StateId>,
    /// Current state in FST being walked (not to be
    /// confused with the thse StateId of the combined FST).
    fst_state: Option<StateId>,
}

impl ReplaceStateTuple {
    fn new(prefix_id: usize, fst_id: Option<StateId>, fst_state: Option<StateId>) -> Self {
        Self {
            prefix_id,
            fst_id,
            fst_state,
        }
    }
}

// TODO: Move this struct into its own file + use it for all implementation starting with determinization
struct StateTable<T: Hash + Eq + Clone> {
    table: RefCell<BiHashMap<StateId, T>>,
}

impl<T: Hash + Eq + Clone> StateTable<T> {
    fn new() -> Self {
        Self {
            table: RefCell::new(BiHashMap::new()),
        }
    }

    /// Looks up integer ID from entry. If it doesn't exist and insert
    fn find_id(&self, tuple: &T) -> StateId {
        if !self.table.borrow().contains_right(tuple) {
            let n = self.table.borrow().len();
            self.table.borrow_mut().insert(n, tuple.clone());
        }
        *self.table.borrow().get_by_right(tuple).unwrap()
    }

    /// Looks up tuple from integer ID.
    fn find_tuple(&self, tuple_id: StateId) -> Ref<T> {
        //        self.table.borrow().get_by_left(&tuple_id).as_ref().unwrap()
        let table = self.table.borrow();
        Ref::map(table, |x| x.get_by_left(&tuple_id).unwrap())
    }
}

struct ReplaceStateTable {
    pub prefix_table: StateTable<ReplaceStackPrefix>,
    pub tuple_table: StateTable<ReplaceStateTuple>,
}

impl ReplaceStateTable {
    fn new() -> Self {
        Self {
            prefix_table: StateTable::new(),
            tuple_table: StateTable::new(),
        }
    }
}
