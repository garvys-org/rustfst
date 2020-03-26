use failure::Fallible;
use serde_derive::{Deserialize, Serialize};

use crate::algorithms::compose_filters::{
    AltSequenceComposeFilter, MatchComposeFilter, NoMatchComposeFilter, NullComposeFilter,
    SequenceComposeFilter, TrivialComposeFilter,
};
use crate::algorithms::matchers::SortedMatcher;
use crate::algorithms::{
    compose, compose_with_config, ComposeConfig, ComposeFilterEnum, ComposeFst,
};
use crate::fst_impls::VectorFst;
use crate::fst_traits::SerializableFst;
use crate::semirings::{SerializableSemiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::algorithms::dynamic_fst::compare_fst_static_dynamic;
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct ComposeOperationResult {
    fst_2: String,
    result: String,
    filter_name: String,
}

pub struct ComposeTestData<F>
where
    F: SerializableFst,
    F::W: SerializableSemiring,
{
    pub fst_2: F,
    pub result: F,
    pub filter_name: String,
}

impl ComposeOperationResult {
    pub fn parse<F>(&self) -> ComposeTestData<F>
    where
        F: SerializableFst,
        F::W: SerializableSemiring,
    {
        ComposeTestData {
            fst_2: F::from_text_string(self.fst_2.as_str()).unwrap(),
            result: F::from_text_string(self.result.as_str()).unwrap(),
            filter_name: self.filter_name.clone(),
        }
    }
}

fn do_test_compose<W>(
    fst_raw: &VectorFst<W>,
    compose_test_data: &ComposeTestData<VectorFst<W>>,
    filter: ComposeFilterEnum,
) -> Fallible<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
{
    let mut config = ComposeConfig::default();
    config.connect = false;
    config.compose_filter = filter;

    let fst_res_static: VectorFst<_> =
        compose_with_config(fst_raw, &compose_test_data.fst_2, config)?;

    assert_eq!(
        compose_test_data.result,
        fst_res_static,
        "{}",
        error_message_fst!(
            compose_test_data.result,
            fst_res_static,
            format!(
                "Compose failed : filter_name = {:?}",
                compose_test_data.filter_name
            )
        )
    );

    Ok(())
}

pub fn test_compose<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
{
    for compose_test_data in &test_data.compose {
        match compose_test_data.filter_name.as_str() {
            "auto" => do_test_compose(
                &test_data.raw,
                compose_test_data,
                ComposeFilterEnum::AutoFilter,
            )?,
            "null" => do_test_compose(
                &test_data.raw,
                compose_test_data,
                ComposeFilterEnum::NullFilter,
            )?,
            "trivial" => do_test_compose(
                &test_data.raw,
                compose_test_data,
                ComposeFilterEnum::TrivialFilter,
            )?,
            "sequence" => do_test_compose(
                &test_data.raw,
                compose_test_data,
                ComposeFilterEnum::SequenceFilter,
            )?,
            "alt_sequence" => do_test_compose(
                &test_data.raw,
                compose_test_data,
                ComposeFilterEnum::SequenceFilter,
            )?,
            "match" => do_test_compose(
                &test_data.raw,
                compose_test_data,
                ComposeFilterEnum::MatchFilter,
            )?,
            "no_match" => do_test_compose(
                &test_data.raw,
                compose_test_data,
                ComposeFilterEnum::NoMatchFilter,
            )?,
            _ => panic!("Not supported : {}", &compose_test_data.filter_name),
        }
    }
    Ok(())
}
