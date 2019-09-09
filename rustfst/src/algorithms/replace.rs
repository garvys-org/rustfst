use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use bimap::BiHashMap;
use failure::{bail, Fallible};

use crate::algorithms::cache::CacheImpl;
use crate::fst_traits::ExpandedFst;
use crate::{Label, StateId, EPS_LABEL};
use std::hash::Hash;

/// This specifies what labels to output on the call or return arc.
#[derive(PartialOrd, PartialEq)]
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

type FstList<F> = Vec<(Label, F)>;

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
    nonterminal_set: HashSet<Label>,
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
            nonterminal_set: HashSet::new(),
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
                    .find_id(&ReplaceStateTuple::new(prefix, self.root, fst_start));
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
                .get(tuple.fst_id)
                .unwrap()
                .final_weight(tuple.fst_state)
                .map(|e| e.cloned())
        } else {
            Ok(None)
        }
    }

    #[allow(unused)]
    fn expand(&mut self, state: StateId) -> Fallible<()> {
        let tuple = self.state_table.tuple_table.find_tuple(state);

//        if tuple.fst_state

        unimplemented!();

        self.cache_impl.mark_expanded(state);
        Ok(())
    }

    fn get_prefix_id(&mut self, prefix: &ReplaceStackPrefix) -> StateId {
        self.state_table.prefix_table.find_id(prefix)
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
        self.push(fst_id, nextstate);
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
    fst_id: StateId,
    /// Current state in FST being walked (not to be
    /// confused with the thse StateId of the combined FST).
    fst_state: StateId,
}

impl ReplaceStateTuple {
    fn new(prefix_id: usize, fst_id: StateId, fst_state: StateId) -> Self {
        Self {
            prefix_id,
            fst_id,
            fst_state,
        }
    }
}

// TODO: Move this struct into its own file + use it for all implementation starting with determinization
struct StateTable<T: Hash + Eq + Clone> {
    table: BiHashMap<StateId, T>,
}

impl<T: Hash + Eq + Clone> StateTable<T> {
    fn new() -> Self {
        Self {
            table: BiHashMap::new(),
        }
    }

    /// Looks up integer ID from entry. If it doesn't exist and insert
    fn find_id(&mut self, tuple: &T) -> StateId {
        if !self.table.contains_right(tuple) {
            let n = self.table.len();
            self.table.insert(n, tuple.clone());
        }
        *self.table.get_by_right(tuple).unwrap()
    }

    /// Looks up tuple from integer ID.
    fn find_tuple(&self, tuple_id: StateId) -> &T {
        self.table.get_by_left(&tuple_id).as_ref().unwrap()
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
