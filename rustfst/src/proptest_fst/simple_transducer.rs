use crate::prelude::{TropicalWeight, VectorFst};
use crate::proptest_fst::{MAX_ILABEL, MAX_OLABEL, MAX_WEIGHT_VALUE};
use crate::utils::transducer;
use crate::{Label, Semiring};
use proptest::arbitrary::any_with;
use proptest::prelude::{Arbitrary, BoxedStrategy, Strategy};

#[derive(Debug, Clone)]
pub struct SimpleTransducer(pub VectorFst<TropicalWeight>);

#[derive(Debug, Clone)]
pub struct ProptestSimpleTransducerConfig {
    pub ilabel: Label,
    pub olabel: Label,
    pub weight_value: usize,
}

impl Default for ProptestSimpleTransducerConfig {
    fn default() -> Self {
        Self {
            ilabel: MAX_ILABEL,
            olabel: MAX_OLABEL,
            weight_value: MAX_WEIGHT_VALUE,
        }
    }
}

impl Arbitrary for ProptestSimpleTransducerConfig {
    type Parameters = ProptestSimpleTransducerConfig;

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        (0..args.ilabel, 0..args.olabel, 0..args.weight_value)
            .prop_map(
                move |(ilabel, olabel, weight_value)| ProptestSimpleTransducerConfig {
                    ilabel,
                    olabel,
                    weight_value,
                },
            )
            .boxed()
    }

    type Strategy = BoxedStrategy<ProptestSimpleTransducerConfig>;
}

impl Arbitrary for SimpleTransducer {
    type Parameters = ProptestSimpleTransducerConfig;

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        any_with::<ProptestSimpleTransducerConfig>(args)
            .prop_map(move |config: ProptestSimpleTransducerConfig| {
                SimpleTransducer(transducer(
                    &[config.ilabel],
                    &[config.olabel],
                    TropicalWeight::new(config.weight_value as f32),
                ))
            })
            .boxed()
    }

    type Strategy = BoxedStrategy<SimpleTransducer>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fst_traits::ExpandedFst;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_proptest_simple_transducer(fst in any::<SimpleTransducer>()) {
            prop_assert!(fst.0.num_states() > 0);
        }
    }
}
