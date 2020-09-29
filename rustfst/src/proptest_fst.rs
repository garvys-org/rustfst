#![cfg(test)]

use proptest::prelude::*;

use crate::fst_impls::VectorFst;
use crate::fst_traits::MutableFst;
use crate::semirings::{Semiring, TropicalWeight};
use crate::Tr;

static MAX_NUM_STATES: usize = 100;
static MAX_ILABEL: usize = 100;
static MAX_OLABEL: usize = 100;
static MAX_NUM_ARCS: usize = 500;

#[derive(Debug, Clone, Copy)]
pub struct ProptestFstConfig {
    pub max_num_states: usize,
    pub max_ilabel: usize,
    pub max_olabel: usize,
    pub max_num_arcs: usize,
}

impl Default for ProptestFstConfig {
    fn default() -> Self {
        Self {
            max_num_states: MAX_NUM_STATES,
            max_ilabel: MAX_ILABEL,
            max_olabel: MAX_OLABEL,
            max_num_arcs: MAX_NUM_ARCS,
        }
    }
}

fn proptest_weight() -> impl Strategy<Value = Option<TropicalWeight>> {
    prop_oneof![
        Just(None),
        (2..10).prop_map(|e| Some(TropicalWeight::new(e as f32)))
    ]
}

fn proptest_trs(nstates: usize, max_ilabel: usize, max_olabel: usize, max_num_arcs: usize) -> impl Strategy<Value = Vec<(usize, Tr<TropicalWeight>)>> {
    proptest::collection::vec(
        (
            0..nstates,
            0..max_ilabel,
            0..max_olabel,
            proptest_weight(),
            0..nstates,
        ),
        // Number of trs
        0..max_num_arcs,
    )
    .prop_map(|v| {
        v.into_iter()
            .map(|(state, ilabel, olabel, weight, nextstate)| {
                (
                    state,
                    Tr {
                        ilabel,
                        olabel,
                        weight: weight.unwrap_or_else(TropicalWeight::one),
                        nextstate,
                    },
                )
            })
            .collect()
    })
}

pub fn proptest_fst_with_config(config: ProptestFstConfig) -> impl Strategy<Value = VectorFst<TropicalWeight>> {
    let nstates_strategy = 1..config.max_num_states;
    nstates_strategy
        .prop_flat_map(move |nstates| {
            (
                // Number of states.
                Just(nstates),
                // Start state.
                (0..nstates),
                // List of states : Vec<State, Tr>.
                proptest_trs(nstates, config.max_ilabel, config.max_olabel, config.max_num_arcs),
                // List of final weight.
                proptest::collection::vec(proptest_weight(), nstates..=nstates),
            )
        })
        .prop_map(|(nstates, start_state, trs, final_weights)| {
            let mut fst = VectorFst::new();

            // Create all states.
            fst.add_states(nstates);

            // Set start state.
            fst.set_start(start_state).unwrap();

            // Add trs.
            for (state, tr) in trs.into_iter() {
                unsafe { fst.add_tr_unchecked(state, tr) };
            }

            // Set final weights.
            for (idx, final_weight) in final_weights.into_iter().enumerate() {
                if let Some(_final_weight) = final_weight {
                    unsafe { fst.set_final_unchecked(idx, _final_weight) };
                }
            }

            fst
        })
}

pub fn proptest_fst() -> impl Strategy<Value = VectorFst<TropicalWeight>> {
    proptest_fst_with_config(ProptestFstConfig::default())
}
