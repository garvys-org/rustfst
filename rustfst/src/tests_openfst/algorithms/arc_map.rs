use std::fmt::Display;

use failure::Fallible;
use pretty_assertions::assert_eq;
use serde_derive::{Deserialize, Serialize};

use crate::algorithms::arc_mappers::{
    IdentityArcMapper, InputEpsilonMapper, InvertWeightMapper, OutputEpsilonMapper, PlusMapper,
    QuantizeMapper, RmWeightMapper, TimesMapper,
};
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::semirings::WeaklyDivisibleSemiring;
use crate::semirings::WeightQuantize;
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct ArcMapWithWeightOperationResult {
    weight: String,
    result: String,
}

pub struct ArcMapWithWeightTestData<F>
where
    F: SerializableFst,
    F::W: SerializableSemiring,
{
    pub weight: F::W,
    pub result: F,
}

impl ArcMapWithWeightOperationResult {
    pub fn parse<F>(&self) -> ArcMapWithWeightTestData<F>
    where
        F: SerializableFst,
        F::W: SerializableSemiring,
    {
        ArcMapWithWeightTestData {
            weight: F::W::parse_text(self.weight.as_str()).unwrap().1,
            result: F::from_text_string(self.result.as_str()).unwrap(),
        }
    }
}

pub fn test_arc_map_identity<F>(test_data: &FstTestData<F>) -> Fallible<()>
where
    F: SerializableFst + MutableFst + Display,
    F::W: SerializableSemiring + WeightQuantize,
{
    // ArcMap IdentityMapper
    let mut fst_arc_map_identity = test_data.raw.clone();
    let mut identity_mapper = IdentityArcMapper {};
    fst_arc_map_identity.arc_map(&mut identity_mapper)?;
    assert_eq!(
        test_data.arc_map_identity,
        fst_arc_map_identity,
        "{}",
        error_message_fst!(
            test_data.arc_map_identity,
            fst_arc_map_identity,
            "ArcMap identity"
        )
    );
    Ok(())
}

pub fn test_arc_map_invert<F>(test_data: &FstTestData<F>) -> Fallible<()>
where
    F: SerializableFst + MutableFst + Display,
    F::W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
{
    // ArcMap InvertWeightMapper
    let mut fst_arc_map_invert = test_data.raw.clone();
    let mut invertweight_mapper = InvertWeightMapper {};
    fst_arc_map_invert.arc_map(&mut invertweight_mapper)?;
    assert_eq!(
        test_data.arc_map_invert,
        fst_arc_map_invert,
        "{}",
        error_message_fst!(
            test_data.arc_map_invert,
            fst_arc_map_invert,
            "ArcMap InvertWeight"
        )
    );
    Ok(())
}

pub fn test_arc_map_input_epsilon<F>(test_data: &FstTestData<F>) -> Fallible<()>
where
    F: SerializableFst + MutableFst + Display,
    F::W: SerializableSemiring + WeightQuantize,
{
    let mut fst_arc_map = test_data.raw.clone();
    let mut mapper = InputEpsilonMapper {};
    fst_arc_map.arc_map(&mut mapper)?;
    assert_eq!(
        test_data.arc_map_input_epsilon,
        fst_arc_map,
        "{}",
        error_message_fst!(
            test_data.arc_map_input_epsilon,
            fst_arc_map,
            "ArcMap InputEpsilonMapper"
        )
    );
    Ok(())
}

pub fn test_arc_map_output_epsilon<F>(test_data: &FstTestData<F>) -> Fallible<()>
where
    F: SerializableFst + MutableFst + Display,
    F::W: SerializableSemiring + WeightQuantize,
{
    let mut fst_arc_map = test_data.raw.clone();
    let mut mapper = OutputEpsilonMapper {};
    fst_arc_map.arc_map(&mut mapper)?;
    assert_eq!(
        test_data.arc_map_output_epsilon,
        fst_arc_map,
        "{}",
        error_message_fst!(
            test_data.arc_map_output_epsilon,
            fst_arc_map,
            "ArcMap OutputEpsilonMapper"
        )
    );
    Ok(())
}

pub fn test_arc_map_plus<F>(test_data: &FstTestData<F>) -> Fallible<()>
where
    F: SerializableFst + MutableFst + Display,
    F::W: SerializableSemiring + WeightQuantize,
{
    let mut fst_arc_map = test_data.raw.clone();
    let mut mapper = PlusMapper::from_weight(test_data.arc_map_plus.weight.clone());
    fst_arc_map.arc_map(&mut mapper)?;
    assert_eq!(
        test_data.arc_map_plus.result,
        fst_arc_map,
        "{}",
        error_message_fst!(
            test_data.arc_map_plus.result,
            fst_arc_map,
            "ArcMap PlusMapper"
        )
    );
    Ok(())
}

pub fn test_arc_map_times<F>(test_data: &FstTestData<F>) -> Fallible<()>
where
    F: SerializableFst + MutableFst + Display,
    F::W: SerializableSemiring + WeightQuantize,
{
    let mut fst_arc_map = test_data.raw.clone();
    let mut mapper = TimesMapper::from_weight(test_data.arc_map_times.weight.clone());
    fst_arc_map.arc_map(&mut mapper)?;
    assert_eq!(
        test_data.arc_map_times.result,
        fst_arc_map,
        "{}",
        error_message_fst!(
            test_data.arc_map_times.result,
            fst_arc_map,
            "ArcMap TimesMapper"
        )
    );
    Ok(())
}

pub fn test_arc_map_quantize<F>(test_data: &FstTestData<F>) -> Fallible<()>
where
    F: SerializableFst + MutableFst + Display,
    F::W: SerializableSemiring + WeightQuantize,
{
    let mut fst_arc_map = test_data.raw.clone();
    let mut mapper = QuantizeMapper {};
    fst_arc_map.arc_map(&mut mapper)?;
    assert_eq!(
        test_data.arc_map_quantize,
        fst_arc_map,
        "{}",
        error_message_fst!(
            test_data.arc_map_quantize,
            fst_arc_map,
            "ArcMap QuantizeMapper"
        )
    );
    Ok(())
}

pub fn test_arc_map_rmweight<F>(test_data: &FstTestData<F>) -> Fallible<()>
where
    F: SerializableFst + MutableFst + Display,
    F::W: SerializableSemiring + WeightQuantize,
{
    // ArcMap RmWeightMapper
    let mut fst_arc_map_rmweight = test_data.raw.clone();
    let mut rmweight_mapper = RmWeightMapper {};
    fst_arc_map_rmweight.arc_map(&mut rmweight_mapper)?;
    assert_eq!(
        test_data.arc_map_rmweight,
        fst_arc_map_rmweight,
        "{}",
        error_message_fst!(
            test_data.arc_map_rmweight,
            fst_arc_map_rmweight,
            "ArcMap RmWeight"
        )
    );
    Ok(())
}
