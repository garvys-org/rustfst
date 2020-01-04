use std::collections::hash_map::Entry;
use std::collections::{BTreeSet, HashMap};
use std::hash::Hash;

use failure::{bail, Fallible};

use crate::algorithms::cache::{CacheImpl, FstImpl, StateTable};
use crate::algorithms::replace::ReplaceLabelType::{ReplaceLabelInput, ReplaceLabelNeither};
use crate::fst_traits::{CoreFst, ExpandedFst, MutableFst};
use crate::semirings::Semiring;
use crate::{Arc, Label, StateId, SymbolTable, EPS_LABEL};

/// This specifies what labels to output on the call or return arc.
#[derive(PartialOrd, PartialEq, Copy, Clone, Debug)]
enum ReplaceLabelType {
    /// Epsilon labels on both input and output.
    ReplaceLabelNeither,
    /// Non-epsilon labels on input and epsilon on output.
    ReplaceLabelInput,
    /// Epsilon on input and non-epsilon on output.
    ReplaceLabelOutput,
    #[allow(unused)]
    /// Non-epsilon labels on both input and output.
    ReplaceLabelBoth,
}

#[derive(PartialOrd, PartialEq, Clone, Debug)]
struct ReplaceFstOptions {
    /// Index of root rule for expansion.
    root: Label,
    /// How to label call arc.
    call_label_type: ReplaceLabelType,
    /// How to label return arc.
    return_label_type: ReplaceLabelType,
    /// Specifies output label to put on call arc; if `None`, use existing label
    /// on call arc. Otherwise, use this field as the output label.
    call_output_label: Option<Label>,
    /// Specifies label to put on return arc.
    return_label: Label,
}

impl ReplaceFstOptions {
    pub fn new(root: Label, epsilon_on_replace: bool) -> Self {
        Self {
            root,
            call_label_type: if epsilon_on_replace {
                ReplaceLabelNeither
            } else {
                ReplaceLabelInput
            },
            return_label_type: ReplaceLabelNeither,
            call_output_label: if epsilon_on_replace { Some(0) } else { None },
            return_label: 0,
        }
    }
}

/// Recursively replaces arcs in the root FSTs with other FSTs.
///
/// Replace supports replacement of arcs in one Fst with another FST. This
/// replacement is recursive. Replace takes an array of FST(s). One FST
/// represents the root (or topology) machine. The root FST refers to other FSTs
/// by recursively replacing arcs labeled as non-terminals with the matching
/// non-terminal FST. Currently Replace uses the output symbols of the arcs to
/// determine whether the arc is a non-terminal arc or not. A non-terminal can be
/// any label that is not a non-zero terminal label in the output alphabet.
///
/// Note that input argument is a vector of pairs. These correspond to the tuple
/// of non-terminal Label and corresponding FST.
pub fn replace<F1, F2>(
    fst_list: Vec<(Label, F1)>,
    root: Label,
    epsilon_on_replace: bool,
) -> Fallible<F2>
where
    F1: ExpandedFst,
    F1::W: 'static,
    F2: MutableFst<W = F1::W> + ExpandedFst<W = F1::W>,
{
    let opts = ReplaceFstOptions::new(root, epsilon_on_replace);
    let mut fst = ReplaceFstImpl::new(fst_list, opts)?;
    fst.compute()
}

/// Returns true if label type on arc results in epsilon input label.
fn epsilon_on_input(label_type: ReplaceLabelType) -> bool {
    label_type == ReplaceLabelType::ReplaceLabelNeither
        || label_type == ReplaceLabelType::ReplaceLabelOutput
}

/// Returns true if label type on arc results in epsilon input label.
fn epsilon_on_output(label_type: ReplaceLabelType) -> bool {
    label_type == ReplaceLabelType::ReplaceLabelNeither
        || label_type == ReplaceLabelType::ReplaceLabelInput
}

#[allow(unused)]
// Necessary when setting the properties.
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

#[derive(Clone, Debug, PartialEq)]
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

impl<F: ExpandedFst> FstImpl<F::W> for ReplaceFstImpl<F>
where
    F::W: 'static,
{
    fn cache_impl_mut(&mut self) -> &mut CacheImpl<<F as CoreFst>::W> {
        &mut self.cache_impl
    }

    fn cache_impl_ref(&self) -> &CacheImpl<<F as CoreFst>::W> {
        &self.cache_impl
    }

    fn expand(&mut self, state: usize) -> Fallible<()> {
        let tuple = self.state_table.tuple_table.find_tuple(state).clone();
        if let Some(fst_state) = tuple.fst_state {
            if let Some(arc) = self.compute_final_arc(state) {
                self.cache_impl.push_arc(state, arc)?;
            }

            for arc in self
                .fst_array
                .get(tuple.fst_id.unwrap())
                .unwrap()
                .arcs_iter(fst_state)?
            {
                if let Some(new_arc) = self.compute_arc(&tuple, arc) {
                    self.cache_impl.push_arc(state, new_arc)?;
                }
            }
        }
        Ok(())
    }

    fn compute_start(&mut self) -> Fallible<Option<usize>> {
        if self.fst_array.is_empty() {
            return Ok(None);
        } else {
            if let Some(fst_start) = self.fst_array[self.root].start() {
                let prefix = self.get_prefix_id(ReplaceStackPrefix::new());
                let start = self.state_table.tuple_table.find_id(ReplaceStateTuple::new(
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

    fn compute_final(&mut self, state: usize) -> Fallible<Option<F::W>> {
        let tuple = self.state_table.tuple_table.find_tuple(state);
        if tuple.prefix_id == 0 {
            self.fst_array
                .get(tuple.fst_id.unwrap())
                .unwrap()
                .final_weight(tuple.fst_state.unwrap())
                .map(|e| e.cloned())
        } else {
            Ok(None)
        }
    }
}

impl<F: ExpandedFst> ReplaceFstImpl<F> {
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

        for (label, fst) in fst_list.into_iter() {
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
            // Necessary to avoid issues with the RefCell.
            let tuple_owned = tuple.clone();
            drop(tuple);
            let tuple = tuple_owned;

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
            let nextstate = self.state_table.tuple_table.find_id(ReplaceStateTuple::new(
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

    fn get_prefix_id(&self, prefix: ReplaceStackPrefix) -> StateId {
        self.state_table.prefix_table.find_id(prefix)
    }

    fn pop_prefix(&self, mut prefix: ReplaceStackPrefix) -> StateId {
        prefix.pop();
        self.get_prefix_id(prefix)
    }

    fn push_prefix(
        &self,
        mut prefix: ReplaceStackPrefix,
        fst_id: Option<Label>,
        nextstate: Option<StateId>,
    ) -> StateId {
        prefix.push(fst_id, nextstate);
        self.get_prefix_id(prefix)
    }

    fn compute_arc<W: Semiring>(&self, tuple: &ReplaceStateTuple, arc: &Arc<W>) -> Option<Arc<W>> {
        if arc.olabel == EPS_LABEL
            || arc.olabel < *self.nonterminal_set.iter().next().unwrap()
            || arc.olabel > *self.nonterminal_set.iter().rev().next().unwrap()
        {
            let state_tuple =
                ReplaceStateTuple::new(tuple.prefix_id, tuple.fst_id, Some(arc.nextstate));
            let nextstate = self.state_table.tuple_table.find_id(state_tuple);
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
                    let nt_nextstate = self.state_table.tuple_table.find_id(
                        ReplaceStateTuple::new(nt_prefix, Some(*nonterminal), Some(nt_start)),
                    );
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
                let nextstate = self.state_table.tuple_table.find_id(ReplaceStateTuple::new(
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
}

#[derive(Hash, Eq, PartialOrd, PartialEq, Clone, Debug)]
struct PrefixTuple {
    fst_id: Option<Label>,
    nextstate: Option<StateId>,
}

#[derive(Hash, Eq, PartialOrd, PartialEq, Clone, Debug)]
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
}

#[derive(Hash, Eq, PartialOrd, PartialEq, Clone, Debug)]
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

#[derive(Clone, Debug, PartialEq)]
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

// Dynamic fst
use crate::fst_traits::{ArcIterator, Fst, StateIterator};
use std::cell::UnsafeCell;
use std::fmt;
use std::rc::Rc;
use std::slice::Iter as IterSlice;

pub struct ReplaceFst<F: ExpandedFst> {
    fst_impl: UnsafeCell<ReplaceFstImpl<F>>,
    isymt: Option<Rc<SymbolTable>>,
    osymt: Option<Rc<SymbolTable>>,
}

impl<F: ExpandedFst> ReplaceFst<F>
where
    F::W: 'static,
{
    pub fn new(fst_list: Vec<(Label, F)>, root: Label, epsilon_on_replace: bool) -> Fallible<Self> {
        let mut isymt = None;
        let mut osymt = None;
        if let Some(first_elt) = fst_list.first() {
            isymt = first_elt.1.input_symbols();
            osymt = first_elt.1.output_symbols();
        }
        let opts = ReplaceFstOptions::new(root, epsilon_on_replace);
        let fst = ReplaceFstImpl::new(fst_list, opts)?;
        Ok(ReplaceFst {
            isymt, osymt,
            fst_impl: UnsafeCell::new(fst),
        })
    }

    fn num_known_states(&self) -> usize {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_ref().unwrap() };
        fst_impl.num_known_states()
    }
}

impl<F: ExpandedFst> PartialEq for ReplaceFst<F>
where
    F::W: 'static,
{
    fn eq(&self, other: &Self) -> bool {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_ref().unwrap() };

        let ptr_other = other.fst_impl.get();
        let fst_impl_other = unsafe { ptr_other.as_ref().unwrap() };

        fst_impl.eq(fst_impl_other)
    }
}

impl<F: ExpandedFst> fmt::Debug for ReplaceFst<F>
where
    F::W: 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_ref().unwrap() };
        write!(f, "ReplaceFst {{ {:?} }}", &fst_impl)
    }
}

impl<F: ExpandedFst + 'static> Clone for ReplaceFst<F>
where
    F::W: 'static,
{
    fn clone(&self) -> Self {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_ref().unwrap() };
        Self {
            fst_impl: UnsafeCell::new(fst_impl.clone()),
            isymt: self.input_symbols(),
            osymt: self.output_symbols()
        }
    }
}

impl<F: ExpandedFst> CoreFst for ReplaceFst<F>
where
    F::W: 'static,
{
    type W = F::W;

    fn start(&self) -> Option<usize> {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_mut().unwrap() };
        fst_impl.start().unwrap()
    }

    fn final_weight(&self, state_id: usize) -> Fallible<Option<&Self::W>> {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_mut().unwrap() };
        fst_impl.final_weight(state_id)
    }

    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<&Self::W> {
        self.final_weight(state_id).unwrap()
    }

    fn num_arcs(&self, s: usize) -> Fallible<usize> {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_mut().unwrap() };
        fst_impl.num_arcs(s)
    }

    unsafe fn num_arcs_unchecked(&self, s: usize) -> usize {
        self.num_arcs(s).unwrap()
    }
}

impl<'a, F: ExpandedFst> ArcIterator<'a> for ReplaceFst<F>
where
    F::W: 'static,
{
    type Iter = IterSlice<'a, Arc<F::W>>;

    fn arcs_iter(&'a self, state_id: usize) -> Fallible<Self::Iter> {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_mut().unwrap() };
        fst_impl.arcs_iter(state_id)
    }

    unsafe fn arcs_iter_unchecked(&'a self, _state_id: usize) -> Self::Iter {
        unimplemented!()
    }
}

#[derive(Clone)]
pub struct StatesIteratorReplaceFst<'a, F: ExpandedFst> {
    fst: &'a ReplaceFst<F>,
    s: usize,
}

impl<'a, F: ExpandedFst> Iterator for StatesIteratorReplaceFst<'a, F>
where
    F::W: 'static,
{
    type Item = StateId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.s < self.fst.num_known_states() {
            let s_cur = self.s;
            // Force expansion of the state
            self.fst.arcs_iter(s_cur).unwrap();
            self.s += 1;
            Some(s_cur)
        } else {
            None
        }
    }
}

impl<'a, F: ExpandedFst + 'a> StateIterator<'a> for ReplaceFst<F>
where
    F::W: 'static,
{
    type Iter = StatesIteratorReplaceFst<'a, F>;

    fn states_iter(&'a self) -> Self::Iter {
        self.start();
        StatesIteratorReplaceFst { fst: &self, s: 0 }
    }
}

impl<'a, F: ExpandedFst + 'static> Fst for ReplaceFst<F>
where
    F::W: 'static,
{
    fn input_symbols(&self) -> Option<Rc<SymbolTable>> {
        self.isymt.clone()
    }

    fn output_symbols(&self) -> Option<Rc<SymbolTable>> {
        self.osymt.clone()
    }
}
