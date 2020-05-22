use std::fmt::Display;

use anyhow::Result;
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};

use crate::algorithms::tr_mappers::{
    IdentityTrMapper, InputEpsilonMapper, InvertWeightMapper, OutputEpsilonMapper, PlusMapper,
    QuantizeMapper, RmWeightMapper, TimesMapper,
};
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::semirings::WeaklyDivisibleSemiring;
use crate::semirings::WeightQuantize;
use crate::tests_openfst::macros::test_eq_fst;
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct TrMapWithWeightOperationResult {
    weight: String,
    result: String,
}

pub struct TrMapWithWeightTestData<W, F>
where
    F: SerializableFst<W>,
    W: SerializableSemiring,
{
    pub weight: W,
    pub result: F,
}

impl TrMapWithWeightOperationResult {
    pub fn parse<W, F>(&self) -> TrMapWithWeightTestData<W, F>
    where
        F: SerializableFst<W>,
        W: SerializableSemiring,
    {
        TrMapWithWeightTestData {
            weight: W::parse_text(self.weight.as_str()).unwrap().1,
            result: F::from_text_string(self.result.as_str()).unwrap(),
        }
    }
}

pub fn test_tr_map_identity<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring + WeightQuantize,
{
    // TrMap IdentityMapper
    let mut fst_tr_map_identity = test_data.raw.clone();
    let mut identity_mapper = IdentityTrMapper {};
    fst_tr_map_identity.tr_map(&mut identity_mapper)?;
    assert_eq!(
        test_data.tr_map_identity,
        fst_tr_map_identity,
        "{}",
        error_message_fst!(
            test_data.tr_map_identity,
            fst_tr_map_identity,
            "TrMap identity"
        )
    );
    Ok(())
}

pub fn test_tr_map_invert<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
{
    // TrMap InvertWeightMapper
    let mut fst_tr_map_invert = test_data.raw.clone();
    let mut invertweight_mapper = InvertWeightMapper {};
    fst_tr_map_invert.tr_map(&mut invertweight_mapper)?;
    assert_eq!(
        test_data.tr_map_invert,
        fst_tr_map_invert,
        "{}",
        error_message_fst!(
            test_data.tr_map_invert,
            fst_tr_map_invert,
            "TrMap InvertWeight"
        )
    );
    Ok(())
}

pub fn test_tr_map_input_epsilon<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring + WeightQuantize,
{
    let mut fst_tr_map = test_data.raw.clone();
    let mut mapper = InputEpsilonMapper {};
    fst_tr_map.tr_map(&mut mapper)?;
    assert_eq!(
        test_data.tr_map_input_epsilon,
        fst_tr_map,
        "{}",
        error_message_fst!(
            test_data.tr_map_input_epsilon,
            fst_tr_map,
            "TrMap InputEpsilonMapper"
        )
    );
    Ok(())
}

pub fn test_tr_map_output_epsilon<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring + WeightQuantize,
{
    let mut fst_tr_map = test_data.raw.clone();
    let mut mapper = OutputEpsilonMapper {};
    fst_tr_map.tr_map(&mut mapper)?;
    assert_eq!(
        test_data.tr_map_output_epsilon,
        fst_tr_map,
        "{}",
        error_message_fst!(
            test_data.tr_map_output_epsilon,
            fst_tr_map,
            "TrMap OutputEpsilonMapper"
        )
    );
    Ok(())
}

pub fn test_tr_map_plus<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring + WeightQuantize,
{
    let mut fst_tr_map = test_data.raw.clone();
    let mut mapper = PlusMapper::from_weight(test_data.tr_map_plus.weight.clone());
    fst_tr_map.tr_map(&mut mapper)?;
    test_eq_fst(
        &test_data.tr_map_plus.result,
        &fst_tr_map,
        "TrMap PlusMapper",
    );
    Ok(())
}

pub fn test_tr_map_times<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring + WeightQuantize,
{
    let mut fst_tr_map = test_data.raw.clone();
    let mut mapper = TimesMapper::from_weight(test_data.tr_map_times.weight.clone());
    fst_tr_map.tr_map(&mut mapper)?;
    test_eq_fst(
        &test_data.tr_map_times.result,
        &fst_tr_map,
        "TrMap TimesMapper",
    );
    Ok(())
}

pub fn test_tr_map_quantize<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring + WeightQuantize,
{
    let mut fst_tr_map = test_data.raw.clone();
    let mut mapper = QuantizeMapper {};
    fst_tr_map.tr_map(&mut mapper)?;

    test_eq_fst(
        &test_data.tr_map_quantize,
        &fst_tr_map,
        "TrMap QuantizeMapper",
    );

    Ok(())
}

pub fn test_tr_map_rmweight<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring + WeightQuantize,
{
    // TrMap RmWeightMapper
    let mut fst_tr_map_rmweight = test_data.raw.clone();
    let mut rmweight_mapper = RmWeightMapper {};
    fst_tr_map_rmweight.tr_map(&mut rmweight_mapper)?;

    test_eq_fst(
        &test_data.tr_map_rmweight,
        &fst_tr_map_rmweight,
        "TrMap RmWeight",
    );
    Ok(())
}
