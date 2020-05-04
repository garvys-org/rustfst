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

fn proptest_weight() -> impl Strategy<Value = Option<TropicalWeight>> {
    prop_oneof![
        Just(None),
        (2..10).prop_map(|e| Some(TropicalWeight::new(e as f32)))
    ]
}

fn proptest_trs(nstates: usize) -> impl Strategy<Value = Vec<(usize, Tr<TropicalWeight>)>> {
    proptest::collection::vec(
        (
            0..nstates,
            0..MAX_ILABEL,
            0..MAX_OLABEL,
            proptest_weight(),
            0..nstates,
        ),
        // Number of trs
        0..MAX_NUM_ARCS,
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

pub(crate) fn proptest_fst() -> impl Strategy<Value = VectorFst<TropicalWeight>> {
    let nstates_strategy = 1..MAX_NUM_STATES;
    nstates_strategy
        .prop_flat_map(|nstates| {
            (
                // Number of states.
                Just(nstates),
                // Start state.
                (0..nstates),
                // List of states : Vec<State, Tr>.
                proptest_trs(nstates),
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
