use std::borrow::Borrow;
use std::collections::hash_map::Entry;
use std::collections::{BTreeSet, HashMap};
use std::hash::Hash;
use std::marker::PhantomData;

use anyhow::{bail, Result};
use itertools::Itertools;

use crate::algorithms::cache::{CacheImpl, FstImpl, StateTable};
use crate::algorithms::dynamic_fst::DynamicFst;
use crate::fst_traits::{CoreFst, ExpandedFst, Fst, MutableFst};
use crate::semirings::Semiring;
use crate::{Tr, Label, StateId, EPS_LABEL};

/// This specifies what labels to output on the call or return arc.
#[derive(PartialOrd, PartialEq, Copy, Clone, Debug, Eq)]
enum ReplaceLabelType {
    /// Epsilon labels on both input and output.
    Neither,
    /// Non-epsilon labels on input and epsilon on output.
    Input,
    /// Epsilon on input and non-epsilon on output.
    Output,
    #[allow(unused)]
    /// Non-epsilon labels on both input and output.
    Both,
}

#[derive(PartialOrd, PartialEq, Clone, Debug, Eq)]
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
                ReplaceLabelType::Neither
            } else {
                ReplaceLabelType::Input
            },
            return_label_type: ReplaceLabelType::Neither,
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
///
/// # Example
///
/// ## Root Fst
///
/// ![replace_in_1](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/replace_in_1.svg?sanitize=true)
///
/// ## Fst for non-terminal #NAME
///
/// ![replace_in_2](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/replace_in_2.svg?sanitize=true)
///
/// ## Fst for non-termincal #FIRSTNAME
///
/// ![replace_in_3](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/replace_in_3.svg?sanitize=true)
///
/// ## Fst for non-termincal #LASTNAME
///
/// ![replace_in_4](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/replace_in_4.svg?sanitize=true)
///
/// ## Output
///
/// ![replace_out](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/replace_out.svg?sanitize=true)
///
pub fn replace<F1, F2, B>(
    fst_list: Vec<(Label, B)>,
    root: Label,
    epsilon_on_replace: bool,
) -> Result<F2>
where
    F1: Fst,
    F1::W: Semiring + 'static,
    F2: MutableFst<W = F1::W> + ExpandedFst<W = F1::W>,
    B: Borrow<F1>,
{
    let opts = ReplaceFstOptions::new(root, epsilon_on_replace);
    let mut fst = ReplaceFstImpl::new(fst_list, opts)?;
    fst.compute()
}

/// Returns true if label type on arc results in epsilon input label.
fn epsilon_on_input(label_type: ReplaceLabelType) -> bool {
    label_type == ReplaceLabelType::Neither || label_type == ReplaceLabelType::Output
}

/// Returns true if label type on arc results in epsilon input label.
fn epsilon_on_output(label_type: ReplaceLabelType) -> bool {
    label_type == ReplaceLabelType::Neither || label_type == ReplaceLabelType::Input
}

#[allow(unused)]
// Necessary when setting the properties.
fn replace_transducer(
    call_label_type: ReplaceLabelType,
    return_label_type: ReplaceLabelType,
    call_output_label: Option<Label>,
) -> bool {
    call_label_type == ReplaceLabelType::Input
        || call_label_type == ReplaceLabelType::Output
        || (call_label_type == ReplaceLabelType::Both && call_output_label.is_some())
        || return_label_type == ReplaceLabelType::Input
        || return_label_type == ReplaceLabelType::Output
}

#[derive(Clone, Eq)]
pub struct ReplaceFstImpl<F: Fst, B: Borrow<F>> {
    cache_impl: CacheImpl<F::W>,
    call_label_type_: ReplaceLabelType,
    return_label_type_: ReplaceLabelType,
    call_output_label_: Option<Label>,
    return_label_: Label,
    fst_array: Vec<B>,
    nonterminal_set: BTreeSet<Label>,
    nonterminal_hash: HashMap<Label, Label>,
    root: Label,
    state_table: ReplaceStateTable,
    fst_type: PhantomData<F>,
}

impl<F: Fst + PartialEq, B: Borrow<F>> PartialEq for ReplaceFstImpl<F, B> {
    fn eq(&self, other: &Self) -> bool {
        self.cache_impl.eq(&other.cache_impl)
            && self.call_label_type_.eq(&other.call_label_type_)
            && self.return_label_type_.eq(&other.return_label_type_)
            && self.call_output_label_.eq(&other.call_output_label_)
            && self.return_label_.eq(&other.return_label_)
            && self
                .fst_array
                .iter()
                .zip(other.fst_array.iter())
                .all(|(a, b)| a.borrow().eq(b.borrow()))
            && self.nonterminal_set.eq(&other.nonterminal_set)
            && self.nonterminal_hash.eq(&other.nonterminal_hash)
            && self.root.eq(&other.root)
            && self.state_table.eq(&other.state_table)
    }
}

impl<F: Fst, B: Borrow<F>> std::fmt::Debug for ReplaceFstImpl<F, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Allocating in debug should be avoided.
        let slice_fst = self.fst_array.iter().map(|fst| fst.borrow()).collect_vec();
        write!(
            f,
            "ReplaceFstImpl {{ cache_impl : {:?}, call_label_type_ : {:?}, \
             return_label_type_ : {:?}, call_output_label_ : {:?}, return_label_ : {:?}, \
             fst_array : {:?}, nonterminal_set : {:?}, nonterminal_hash : {:?}, root : {:?}, \
             state_table : {:?} }}",
            self.cache_impl,
            self.call_label_type_,
            self.return_label_type_,
            self.call_output_label_,
            self.return_label_,
            slice_fst,
            self.nonterminal_set,
            self.nonterminal_hash,
            self.root,
            self.state_table
        )
    }
}

impl<'a, F: Fst, B: Borrow<F>> FstImpl for ReplaceFstImpl<F, B>
where
    F::W: 'static,
{
    type W = F::W;
    fn cache_impl_mut(&mut self) -> &mut CacheImpl<<F as CoreFst>::W> {
        &mut self.cache_impl
    }

    fn cache_impl_ref(&self) -> &CacheImpl<<F as CoreFst>::W> {
        &self.cache_impl
    }

    fn expand(&mut self, state: usize) -> Result<()> {
        let tuple = self.state_table.tuple_table.find_tuple(state).clone();
        if let Some(fst_state) = tuple.fst_state {
            if let Some(arc) = self.compute_final_tr(state) {
                self.cache_impl.push_tr(state, arc)?;
            }

            for arc in self
                .fst_array
                .get(tuple.fst_id.unwrap())
                .unwrap()
                .borrow()
                .arcs_iter(fst_state)?
            {
                if let Some(new_tr) = self.compute_tr(&tuple, arc) {
                    self.cache_impl.push_tr(state, new_tr)?;
                }
            }
        }
        Ok(())
    }

    fn compute_start(&mut self) -> Result<Option<usize>> {
        if self.fst_array.is_empty() {
            Ok(None)
        } else if let Some(fst_start) = self.fst_array[self.root].borrow().start() {
            let prefix = self.get_prefix_id(ReplaceStackPrefix::new());
            let start = self.state_table.tuple_table.find_id(ReplaceStateTuple::new(
                prefix,
                Some(self.root),
                Some(fst_start),
            ));
            Ok(Some(start))
        } else {
            Ok(None)
        }
    }

    fn compute_final(&mut self, state: usize) -> Result<Option<F::W>> {
        let tuple = self.state_table.tuple_table.find_tuple(state);
        if tuple.prefix_id == 0 {
            self.fst_array
                .get(tuple.fst_id.unwrap())
                .unwrap()
                .borrow()
                .final_weight(tuple.fst_state.unwrap())
                .map(|e| e.cloned())
        } else {
            Ok(None)
        }
    }
}

impl<F: Fst, B: Borrow<F>> ReplaceFstImpl<F, B> {
    fn new(fst_list: Vec<(Label, B)>, opts: ReplaceFstOptions) -> Result<Self> {
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
            fst_type: PhantomData,
        };

        if let Some(v) = replace_fst_impl.call_output_label_ {
            if v == EPS_LABEL {
                replace_fst_impl.call_label_type_ = ReplaceLabelType::Neither;
            }
        }

        if replace_fst_impl.return_label_ == 0 {
            replace_fst_impl.return_label_type_ = ReplaceLabelType::Neither;
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

    fn compute_final_tr(&mut self, state: StateId) -> Option<Tr<F::W>> {
        let tuple = self.state_table.tuple_table.find_tuple(state);
        let fst_state = tuple.fst_state?;
        if self
            .fst_array
            .get(tuple.fst_id.unwrap())
            .unwrap()
            .borrow()
            .is_final(fst_state)
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
                .unwrap()
                .borrow()
                .final_weight(fst_state)
                .unwrap()
            {
                return Some(Tr::new(ilabel, olabel, weight.clone(), nextstate));
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

    fn compute_tr<W: Semiring>(&self, tuple: &ReplaceStateTuple, arc: &Tr<W>) -> Option<Tr<W>> {
        if arc.olabel == EPS_LABEL
            || arc.olabel < *self.nonterminal_set.iter().next().unwrap()
            || arc.olabel > *self.nonterminal_set.iter().rev().next().unwrap()
        {
            let state_tuple =
                ReplaceStateTuple::new(tuple.prefix_id, tuple.fst_id, Some(arc.nextstate));
            let nextstate = self.state_table.tuple_table.find_id(state_tuple);
            Some(Tr::new(
                arc.ilabel,
                arc.olabel,
                arc.weight.clone(),
                nextstate,
            ))
        } else {
            // Checks for non-terminal
            if let Some(nonterminal) = self.nonterminal_hash.get(&arc.olabel) {
                let p = self
                    .state_table
                    .prefix_table
                    .find_tuple(tuple.prefix_id)
                    .clone();
                let nt_prefix = self.push_prefix(p, tuple.fst_id, Some(arc.nextstate));
                if let Some(nt_start) = self.fst_array.get(*nonterminal).unwrap().borrow().start() {
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
                    Some(Tr::new(ilabel, olabel, arc.weight.clone(), nt_nextstate))
                } else {
                    None
                }
            } else {
                let nextstate = self.state_table.tuple_table.find_id(ReplaceStateTuple::new(
                    tuple.prefix_id,
                    tuple.fst_id,
                    Some(arc.nextstate),
                ));
                Some(Tr::new(
                    arc.ilabel,
                    arc.olabel,
                    arc.weight.clone(),
                    nextstate,
                ))
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

#[derive(Clone, Debug, PartialEq, Eq)]
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

/// ReplaceFst supports dynamic replacement of arcs in one FST with another FST.
/// This replacement is recursive. ReplaceFst can be used to support a variety of
/// delayed constructions such as recursive transition networks, union, or closure.
pub type ReplaceFst<F, B> = DynamicFst<ReplaceFstImpl<F, B>>;

impl<F: Fst, B: Borrow<F>> ReplaceFst<F, B>
where
    F::W: 'static,
    B: 'static,
{
    pub fn new(fst_list: Vec<(Label, B)>, root: Label, epsilon_on_replace: bool) -> Result<Self> {
        let mut isymt = None;
        let mut osymt = None;
        if let Some(first_elt) = fst_list.first() {
            isymt = first_elt.1.borrow().input_symbols();
            osymt = first_elt.1.borrow().output_symbols();
        }
        let opts = ReplaceFstOptions::new(root, epsilon_on_replace);
        let fst = ReplaceFstImpl::new(fst_list, opts)?;
        Ok(Self::from_impl(fst, isymt, osymt))
    }
}
