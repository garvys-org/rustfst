use ::proptest::prelude::{any_with, Arbitrary, BoxedStrategy, Just, Strategy};
use ::proptest::prop_oneof;

use crate::algorithms::{concat::concat, union::union};
use crate::prelude::{TropicalWeight, VectorFst};
use crate::proptest_fst::simple_transducer::{ProptestSimpleTransducerConfig, SimpleTransducer};
use crate::proptest_fst::{MAX_ILABEL, MAX_NUM_OPERATIONS, MAX_OLABEL, MAX_WEIGHT_VALUE};
use crate::utils::epsilon_machine;
use crate::Label;

#[derive(Debug, Clone, Copy)]
pub struct ProptestFstConfig {
    pub n_operations: usize,
    pub max_ilabel: Label,
    pub max_olabel: Label,
    pub max_weight_value: usize,
}

#[derive(Debug, Clone)]
pub enum Operation {
    Union,
    Concat,
}

impl Arbitrary for Operation {
    type Parameters = ();

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop_oneof![Just(Operation::Union), Just(Operation::Concat),].boxed()
    }

    type Strategy = BoxedStrategy<Operation>;
}

impl Default for ProptestFstConfig {
    fn default() -> Self {
        ProptestFstConfig {
            n_operations: MAX_NUM_OPERATIONS,
            max_ilabel: MAX_ILABEL,
            max_olabel: MAX_OLABEL,
            max_weight_value: MAX_WEIGHT_VALUE,
        }
    }
}

impl Arbitrary for ProptestFstConfig {
    type Parameters = ProptestFstConfig;
    fn arbitrary_with(maxes: ProptestFstConfig) -> Self::Strategy {
        (
            1..maxes.n_operations,
            1..maxes.max_ilabel,
            1..maxes.max_olabel,
            1..maxes.max_weight_value,
        )
            .prop_map(
                |(n_operations, max_ilabel, max_olabel, max_weight_value)| Self {
                    n_operations,
                    max_ilabel,
                    max_olabel,
                    max_weight_value,
                },
            )
            .boxed()
    }
    type Strategy = BoxedStrategy<ProptestFstConfig>;
}

impl Arbitrary for VectorFst<TropicalWeight> {
    type Parameters = ProptestFstConfig;
    fn arbitrary_with(maxes: Self::Parameters) -> Self::Strategy {
        any_with::<ProptestFstConfig>(maxes)
            .prop_flat_map(move |config: ProptestFstConfig| {
                (
                    // List of operations
                    proptest::collection::vec(any_with::<Operation>(()), config.n_operations),
                    // List of dummy transducers
                    proptest::collection::vec(
                        any_with::<SimpleTransducer>(ProptestSimpleTransducerConfig {
                            ilabel: config.max_ilabel,
                            olabel: config.max_olabel,
                            weight_value: config.max_weight_value,
                        }),
                        config.n_operations,
                    ),
                )
            })
            .prop_map(move |(operations, dummy_fsts)| {
                let mut fst: VectorFst<TropicalWeight> = epsilon_machine().unwrap();

                for i in 0..operations.len() {
                    match operations[i] {
                        Operation::Union => union(&mut fst, &dummy_fsts[i].0).unwrap(),
                        Operation::Concat => concat(&mut fst, &dummy_fsts[i].0).unwrap(),
                    };
                }

                fst
            })
            .boxed()
    }
    type Strategy = BoxedStrategy<VectorFst<TropicalWeight>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fst_traits::ExpandedFst;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_proptest_vector_fst(fst in any::<VectorFst<TropicalWeight>>()) {
            prop_assert!(fst.num_states() > 0);
        }
    }
}
