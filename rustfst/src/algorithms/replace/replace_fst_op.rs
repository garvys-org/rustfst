use std::borrow::Borrow;
use std::collections::hash_map::Entry;
use std::collections::{BTreeSet, HashMap};
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;
use itertools::Itertools;

use crate::algorithms::lazy::FstOp;
use crate::algorithms::replace::config::{ReplaceFstOptions, ReplaceLabelType};
use crate::algorithms::replace::state_table::{
    ReplaceStackPrefix, ReplaceStateTable, ReplaceStateTuple,
};
use crate::algorithms::replace::utils::{epsilon_on_input, epsilon_on_output};
use crate::fst_properties::FstProperties;
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{Label, StateId, Tr, Trs, TrsVec, EPS_LABEL};

pub struct ReplaceFstOp<W: Semiring, F: Fst<W>, B: Borrow<F>> {
    call_label_type_: ReplaceLabelType,
    return_label_type_: ReplaceLabelType,
    call_output_label_: Option<Label>,
    return_label_: Label,
    fst_array: Vec<B>,
    nonterminal_set: BTreeSet<Label>,
    nonterminal_hash: HashMap<Label, Label>,
    root: Label,
    state_table: ReplaceStateTable,
    properties: FstProperties,
    fst_type: PhantomData<F>,
    w: PhantomData<W>,
}

impl<W: Semiring, F: Fst<W>, B: Borrow<F>> std::fmt::Debug for ReplaceFstOp<W, F, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Allocating in debug should be avoided.
        let slice_fst = self.fst_array.iter().map(|fst| fst.borrow()).collect_vec();
        write!(
            f,
            "ReplaceFstImpl {{ call_label_type_ : {:?}, \
             return_label_type_ : {:?}, call_output_label_ : {:?}, return_label_ : {:?}, \
             fst_array : {:?}, nonterminal_set : {:?}, nonterminal_hash : {:?}, root : {:?}, \
             state_table : {:?} }}",
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

impl<W: Semiring, F: Fst<W>, B: Borrow<F>> FstOp<W> for ReplaceFstOp<W, F, B> {
    fn compute_start(&self) -> Result<Option<StateId>> {
        if self.fst_array.is_empty() {
            Ok(None)
        } else if let Some(fst_start) = self.fst_array[self.root as usize].borrow().start() {
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

    fn compute_trs(&self, state: StateId) -> Result<TrsVec<W>> {
        let tuple = self.state_table.tuple_table.find_tuple(state);
        let mut trs = vec![];
        if let Some(fst_state) = tuple.fst_state {
            if let Some(tr) = self.compute_final_tr(state) {
                // self.cache_impl.push_tr(state, tr)?;
                trs.push(tr);
            }

            for tr in self
                .fst_array
                .get(tuple.fst_id.unwrap() as usize)
                .unwrap()
                .borrow()
                .get_trs(fst_state)?
                .trs()
            {
                if let Some(new_tr) = self.compute_tr(&tuple, tr) {
                    // self.cache_impl.push_tr(state, new_tr)?;
                    trs.push(new_tr);
                }
            }
        }
        Ok(TrsVec(Arc::new(trs)))
    }

    fn compute_final_weight(&self, state: StateId) -> Result<Option<W>> {
        let tuple = self.state_table.tuple_table.find_tuple(state);
        if tuple.prefix_id == 0 {
            self.fst_array
                .get(tuple.fst_id.unwrap() as usize)
                .unwrap()
                .borrow()
                .final_weight(tuple.fst_state.unwrap())
        } else {
            Ok(None)
        }
    }

    fn properties(&self) -> FstProperties {
        self.properties
    }
}

fn replace_properties<W, F, B>(
    root_label: Label,
    fst_list: &[(Label, B)],
    call_label_type: ReplaceLabelType,
    return_label_type: ReplaceLabelType,
    call_output_label: Option<Label>,
    sorted_and_non_empty: &mut bool,
) -> FstProperties
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F>,
{
    let mut inprops = Vec::with_capacity(fst_list.len());
    let mut all_ilabel_sorted = true;
    let mut all_olabel_sorted = true;
    let mut all_non_empty = true;
    // All nonterminals are negative?
    let all_negative = false; // usize so all positive
                              // All nonterminals are positive and form a dense range containing 1?
    let mut dense_range = true;
    let mut root_fst_idx: usize = 0;
    for i in 0..fst_list.len() {
        let label = fst_list[i].0;
        if (label as usize) > fst_list.len() {
            dense_range = false;
        }
        if label == root_label {
            root_fst_idx = i;
        }
        let fst = &fst_list[i].1;
        if fst.borrow().start().is_none() {
            all_non_empty = false;
        }
        if !fst
            .borrow()
            .properties()
            .contains(FstProperties::I_LABEL_SORTED)
        {
            all_ilabel_sorted = false;
        }
        if !fst
            .borrow()
            .properties()
            .contains(FstProperties::O_LABEL_SORTED)
        {
            all_olabel_sorted = false;
        }
        inprops.push(fst.borrow().properties());
    }
    let props = crate::fst_properties::mutable_properties::replace_properties(
        &inprops,
        root_fst_idx as Label,
        epsilon_on_input(call_label_type),
        epsilon_on_input(return_label_type),
        epsilon_on_output(call_label_type),
        epsilon_on_output(return_label_type),
        replace_transducer(call_label_type, return_label_type, call_output_label),
        all_non_empty,
        all_ilabel_sorted,
        all_olabel_sorted,
        all_negative || dense_range,
    );
    let sorted = props.contains(FstProperties::I_LABEL_SORTED | FstProperties::O_LABEL_SORTED);
    *sorted_and_non_empty = all_non_empty && sorted;
    props
}

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

impl<W: Semiring, F: Fst<W>, B: Borrow<F>> ReplaceFstOp<W, F, B> {
    pub fn new(fst_list: Vec<(Label, B)>, opts: ReplaceFstOptions) -> Result<Self> {
        let mut all_non_empty_and_sorted = false;
        let properties = replace_properties(
            opts.root,
            fst_list.as_slice(),
            opts.call_label_type,
            opts.return_label_type,
            opts.call_output_label,
            &mut all_non_empty_and_sorted,
        );

        let mut replace_fst_impl = Self {
            call_label_type_: opts.call_label_type,
            return_label_type_: opts.return_label_type,
            call_output_label_: opts.call_output_label,
            return_label_: opts.return_label,
            fst_array: Vec::with_capacity(fst_list.len()),
            nonterminal_set: BTreeSet::new(),
            nonterminal_hash: HashMap::new(),
            root: 0,
            state_table: ReplaceStateTable::new(),
            properties,
            fst_type: PhantomData,
            w: PhantomData,
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
                .insert(label, replace_fst_impl.fst_array.len() as StateId);
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

    fn compute_final_tr(&self, state: StateId) -> Option<Tr<W>> {
        let tuple = self.state_table.tuple_table.find_tuple(state);
        let fst_state = tuple.fst_state?;
        if self
            .fst_array
            .get(tuple.fst_id.unwrap() as usize)
            .unwrap()
            .borrow()
            .is_final(fst_state)
            .unwrap()
            && tuple.prefix_id > 0
        {
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
            let stack = self.state_table.prefix_table.find_tuple(tuple.prefix_id);
            let top = stack.top();
            let prefix_id = self.pop_prefix(stack.clone());
            let nextstate = self.state_table.tuple_table.find_id(ReplaceStateTuple::new(
                prefix_id,
                top.fst_id,
                top.nextstate,
            ));
            if let Some(weight) = self
                .fst_array
                .get(tuple.fst_id.unwrap() as usize)
                .unwrap()
                .borrow()
                .final_weight(fst_state)
                .unwrap()
            {
                return Some(Tr::new(ilabel, olabel, weight, nextstate));
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

    fn compute_tr(&self, tuple: &ReplaceStateTuple, tr: &Tr<W>) -> Option<Tr<W>> {
        if tr.olabel == EPS_LABEL
            || tr.olabel < *self.nonterminal_set.iter().next().unwrap()
            || tr.olabel > *self.nonterminal_set.iter().next_back().unwrap()
        {
            let state_tuple =
                ReplaceStateTuple::new(tuple.prefix_id, tuple.fst_id, Some(tr.nextstate));
            let nextstate = self.state_table.tuple_table.find_id(state_tuple);
            Some(Tr::new(tr.ilabel, tr.olabel, tr.weight.clone(), nextstate))
        } else {
            // Checks for non-terminal
            if let Some(nonterminal) = self.nonterminal_hash.get(&tr.olabel) {
                let p = self.state_table.prefix_table.find_tuple(tuple.prefix_id);
                let nt_prefix = self.push_prefix(p, tuple.fst_id, Some(tr.nextstate));
                if let Some(nt_start) = self
                    .fst_array
                    .get(*nonterminal as usize)
                    .unwrap()
                    .borrow()
                    .start()
                {
                    let nt_nextstate = self.state_table.tuple_table.find_id(
                        ReplaceStateTuple::new(nt_prefix, Some(*nonterminal), Some(nt_start)),
                    );
                    let ilabel = if epsilon_on_input(self.call_label_type_) {
                        0
                    } else {
                        tr.ilabel
                    };
                    let olabel = if epsilon_on_output(self.call_label_type_) {
                        0
                    } else {
                        self.call_output_label_.unwrap_or(tr.olabel)
                    };
                    Some(Tr::new(ilabel, olabel, tr.weight.clone(), nt_nextstate))
                } else {
                    None
                }
            } else {
                let nextstate = self.state_table.tuple_table.find_id(ReplaceStateTuple::new(
                    tuple.prefix_id,
                    tuple.fst_id,
                    Some(tr.nextstate),
                ));
                Some(Tr::new(tr.ilabel, tr.olabel, tr.weight.clone(), nextstate))
            }
        }
    }
}
