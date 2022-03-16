#![cfg(test)]

use proptest::prelude::*;

use crate::fst_impls::VectorFst;
use crate::fst_traits::MutableFst;
use crate::semirings::{Semiring, TropicalWeight};
use crate::{Label, StateId, Tr};
use crate::utils::{epsilon_machine, transducer};
use crate::algorithms::union::union;
use crate::algorithms::concat::concat;

static MAX_NUM_OPERATIONS: usize = 30;
static MAX_ILABEL: usize = 10;
static MAX_OLABEL: usize = 10;

#[derive(Debug, Clone, Copy)]
pub struct ProptestFstConfig {
    pub n_operations: usize,
    pub max_ilabel: usize,
    pub max_olabel: usize
}

#[derive(Debug, Clone)]
pub enum Operation {
    Union,
    Concat,
}

impl Arbitrary for Operation {
    type Parameters = ();

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        prop_oneof![
        Just(Operation::Union),
        Just(Operation::Concat)
    ].boxed()
    }

    type Strategy = BoxedStrategy<Operation>;
}


impl Default for ProptestFstConfig {
    fn default() -> Self {
        ProptestFstConfig {
            n_operations: MAX_NUM_OPERATIONS,
            max_ilabel: MAX_ILABEL,
            max_olabel: MAX_OLABEL
        }
    }
}

impl Arbitrary for ProptestFstConfig {
    type Parameters = ProptestFstConfig;
    fn arbitrary_with(maxes: ProptestFstConfig) -> Self::Strategy {
        (
            1..maxes.n_operations,
            1..maxes.max_ilabel,
            1..maxes.max_olabel
        )
            .prop_map(|(n_operations, max_ilabel, max_olabel)| Self {
                n_operations,
                max_ilabel,
                max_olabel,
            })
            .boxed()
    }
    type Strategy = BoxedStrategy<ProptestFstConfig>;
}

impl Arbitrary for TropicalWeight {
    type Parameters = ();
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (1..10).prop_map(|e| TropicalWeight::new(e as f32)).boxed()
    }
    type Strategy = BoxedStrategy<TropicalWeight>;
}

// impl<W: Arbitrary + Semiring> Arbitrary for Tr<W> {
//     type Parameters = ProptestFstConfig;
//     fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
//         (
//             0..args.nstates,
//             0..args.max_ilabel,
//             0..args.max_olabel,
//             any::<W>(),
//         )
//             .prop_map(move |(nextstate, ilabel, olabel, weight)| Tr {
//                 nextstate: nextstate as StateId,
//                 ilabel: ilabel as Label,
//                 olabel: olabel as Label,
//                 weight,
//             })
//             .boxed()
//     }
//     type Strategy = BoxedStrategy<Tr<W>>;
// }

impl Arbitrary for VectorFst<TropicalWeight> {
    type Parameters = ProptestFstConfig;
    fn arbitrary_with(maxes: Self::Parameters) -> Self::Strategy {
        any_with::<ProptestFstConfig>(maxes)
            .prop_flat_map(move |config| {
                (
                    // List of operations
                    proptest::collection::vec(
                        any_with::<Operation>(()),
                        config.n_operations
                    ),
                    // List of ilabels
                    proptest::collection::vec(
                        1..maxes.max_ilabel,
                        config.n_operations
                    ),
                    // List of olabels
                    proptest::collection::vec(
                        1..maxes.max_olabel,
                        config.n_operations,
                    ),
                    // List of weights
                    proptest::collection::vec(
                        any_with::<TropicalWeight>(()),
                        config.n_operations
                    )
                )
            })
            .prop_map(move |(operations, ilabels, olabels, weights)| {

                let mut fst : VectorFst<TropicalWeight> = epsilon_machine().unwrap();

                for i in 0..operations.len() {
                    let dummy_fst : VectorFst<_> = transducer(&[ilabels[i]], &[olabels[i]], weights[i]);
                    match operations[i] {
                        Operation::Union => union(&mut fst, &dummy_fst).unwrap(),
                        Operation::Concat => concat(&mut fst, &dummy_fst).unwrap()
                    };
                }

                fst
            })
            .boxed()
    }
    type Strategy = BoxedStrategy<VectorFst<TropicalWeight>>;
}
