use failure::Fallible;
use serde_derive::{Deserialize, Serialize};

use crate::algorithms::lookahead_matchers::LabelLookAheadMatcher;
use crate::algorithms::matchers::SortedMatcher;
use crate::algorithms::{compose_with_config, ComposeConfig, ComposeFilterEnum};
use crate::fst_impls::{ConstFst, VectorFst};
use crate::fst_traits::SerializableFst;
use crate::semirings::{SerializableSemiring, WeaklyDivisibleSemiring, WeightQuantize};
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

#[allow(unused)]
fn do_test_compose_lookahead<W>(
    fst_raw: &VectorFst<W>,
    compose_test_data: &ComposeTestData<VectorFst<W>>,
) -> Fallible<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
{
    let fst1: ConstFst<_> = fst_raw.clone().into();
    let fst2: VectorFst<_> = compose_test_data.fst_2.clone();

    // type MATCHER1<'a, S> = LabelLookAheadMatcher<'a, S, SortedMatcher<'a, ConstFst<S>>>;
    // type MATCHER2<'a, S> = SortedMatcher<'a, VectorFst<S>>;
    //
    //     type SEQFILTER<'a, 'b, S> =
    //         AltSequenceComposeFilter<'b, VectorFst<S>, MATCHER1<'a, S>, MATCHER2<'b, S>>;
    //     type LOOKFILTER<'a, 'b, S> =
    //         LookAheadComposeFilter<'a, 'b, S, SEQFILTER<'a, 'b, S>, SMatchOutput>;
    //     // type PUSHLABELSFILTER<'a, 'b, S> =
    //     //     PushLabelsComposeFilter<'a, 'b, S, LOOKFILTER<'a, 'b, S>, SMatchOutput>;
    //
    //     let composed_fst = ComposeFst::<'fst1, 'fst2, W, LOOKFILTER<'fst1, 'fst2, W>>::new(fst1, fst2)?;

    unimplemented!()
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
            "lookahead" => do_test_compose_lookahead(&test_data.raw, compose_test_data)?,
            _ => panic!("Not supported : {}", &compose_test_data.filter_name),
        }
    }
    Ok(())
}
