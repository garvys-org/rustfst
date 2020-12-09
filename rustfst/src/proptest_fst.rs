#![cfg(test)]

use proptest::prelude::*;

use crate::fst_impls::VectorFst;
use crate::fst_traits::MutableFst;
use crate::semirings::{Semiring, TropicalWeight};
use crate::{Label, StateId, Tr};

static MAX_NUM_STATES: usize = 10;
static MAX_ILABEL: usize = 10;
static MAX_OLABEL: usize = 10;
static MAX_NUM_ARCS: usize = 50;

#[derive(Debug, Clone, Copy)]
pub struct ProptestFstConfig {
    pub nstates: usize,
    pub max_ilabel: usize,
    pub max_olabel: usize,
    pub max_num_arcs: usize,
}

impl Default for ProptestFstConfig {
    fn default() -> Self {
        ProptestFstConfig {
            nstates: MAX_NUM_STATES,
            max_ilabel: MAX_ILABEL,
            max_olabel: MAX_OLABEL,
            max_num_arcs: MAX_NUM_ARCS,
        }
    }
}

impl Arbitrary for ProptestFstConfig {
    type Strategy = BoxedStrategy<ProptestFstConfig>;
    type Parameters = ProptestFstConfig;
    fn arbitrary_with(maxes: ProptestFstConfig) -> Self::Strategy {
        (
            1..maxes.nstates,
            1..maxes.max_ilabel,
            1..maxes.max_olabel,
            1..maxes.max_num_arcs,
        )
            .prop_map(|(nstates, max_ilabel, max_olabel, max_num_arcs)| Self {
                nstates,
                max_ilabel,
                max_olabel,
                max_num_arcs,
            })
            .boxed()
    }
}

impl Arbitrary for TropicalWeight {
    type Strategy = BoxedStrategy<TropicalWeight>;
    type Parameters = ();
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (1..10).prop_map(|e| TropicalWeight::new(e as f32)).boxed()
    }
}

impl<W: Arbitrary + Semiring> Arbitrary for Tr<W> {
    type Strategy = BoxedStrategy<Tr<W>>;
    type Parameters = ProptestFstConfig;
    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        (
            0..args.nstates,
            0..args.max_ilabel,
            0..args.max_olabel,
            any::<W>(),
        )
            .prop_map(move |(nextstate, ilabel, olabel, weight)| Tr {
                nextstate: nextstate as StateId,
                ilabel: ilabel as Label,
                olabel: olabel as Label,
                weight,
            })
            .boxed()
    }
}

impl<W: Arbitrary + Semiring> Arbitrary for VectorFst<W> {
    type Strategy = BoxedStrategy<VectorFst<W>>;
    type Parameters = ProptestFstConfig;
    fn arbitrary_with(maxes: Self::Parameters) -> Self::Strategy {
        any_with::<ProptestFstConfig>(maxes)
            .prop_flat_map(|config| {
                (
                    // List of states : Vec<Vec<Tr>>
                    proptest::collection::vec(
                        proptest::collection::vec(
                            any_with::<Tr<W>>(config),
                            0..config.max_num_arcs,
                        ),
                        config.nstates..=config.nstates,
                    ),
                    // Start state.
                    (0..config.nstates),
                    // List of final weight.
                    proptest::collection::vec(
                        prop_oneof!(Just(None), any::<W>().prop_map(Some)),
                        config.nstates..=config.nstates,
                    ),
                )
            })
            .prop_map(move |(states, start_state, final_weights)| {
                let mut fst = VectorFst::new();

                // Create all states.
                fst.add_states(states.len());

                // Set start state.
                fst.set_start(start_state as StateId).unwrap();

                // Add trs.
                for (state, trs) in states.into_iter().enumerate() {
                    for tr in trs {
                        unsafe { fst.add_tr_unchecked(state as StateId, tr) };
                    }
                }

                // Set final weights.
                for (idx, final_weight) in final_weights.into_iter().enumerate() {
                    if let Some(_final_weight) = final_weight {
                        unsafe { fst.set_final_unchecked(idx as StateId, _final_weight) };
                    }
                }

                fst
            })
            .boxed()
    }
}
