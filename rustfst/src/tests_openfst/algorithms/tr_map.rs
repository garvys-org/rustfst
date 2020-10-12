use std::fmt::Display;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::algorithms::tr_mappers::{
    IdentityTrMapper, InputEpsilonMapper, InvertWeightMapper, OutputEpsilonMapper, PlusMapper,
    QuantizeMapper, RmWeightMapper, TimesMapper,
};
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::semirings::WeaklyDivisibleSemiring;
use crate::semirings::WeightQuantize;
use crate::tests_openfst::utils::test_eq_fst;
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct TrMapWithWeightOperationResult {
    weight: String,
    result_path: String,
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
    pub fn parse<W, F, P>(&self, dir_path: P) -> TrMapWithWeightTestData<W, F>
    where
        F: SerializableFst<W>,
        W: SerializableSemiring,
        P: AsRef<Path>,
    {
        TrMapWithWeightTestData {
            weight: W::parse_text(self.weight.as_str()).unwrap().1,
            result: F::read(dir_path.as_ref().join(&self.result_path)).unwrap(),
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
    test_eq_fst(
        &test_data.tr_map_identity,
        &fst_tr_map_identity,
        "TrMap Identity",
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
    test_eq_fst(
        &test_data.tr_map_invert,
        &fst_tr_map_invert,
        "TrMap InvertWeight",
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
    test_eq_fst(
        &test_data.tr_map_input_epsilon,
        &fst_tr_map,
        "TrMap InputEpsilonMapper",
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
    test_eq_fst(
        &test_data.tr_map_output_epsilon,
        &fst_tr_map,
        "TrMap OutputEpsilonMapper",
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
    let mut mapper = QuantizeMapper::default();
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
