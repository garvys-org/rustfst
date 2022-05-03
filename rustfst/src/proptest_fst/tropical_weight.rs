use crate::proptest_fst::MAX_WEIGHT_VALUE;
use crate::semirings::TropicalWeight;
use crate::Semiring;
use proptest::arbitrary::Arbitrary;
use proptest::prelude::Strategy;
use proptest::strategy::BoxedStrategy;

#[derive(Debug, Clone)]
pub struct ProptestTropicalWeightConfig(usize);

impl Default for ProptestTropicalWeightConfig {
    fn default() -> Self {
        Self(MAX_WEIGHT_VALUE)
    }
}

impl Arbitrary for TropicalWeight {
    // Max weight value
    type Parameters = ProptestTropicalWeightConfig;

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        (0..args.0)
            .prop_map(move |weight_value| TropicalWeight::new(weight_value as f32))
            .boxed()
    }

    type Strategy = BoxedStrategy<TropicalWeight>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_proptest_tropical_weight(tr in any::<TropicalWeight>()) {
            prop_assert!(*tr.value() >= 0.0 && *tr.value() < MAX_WEIGHT_VALUE as f32);
        }
    }
}
