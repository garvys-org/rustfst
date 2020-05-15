use std::sync::Arc;

use anyhow::Result;
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};

use crate::algorithms::compose::compose_filters::{AltSequenceComposeFilter, ComposeFilter};
use crate::algorithms::compose::lookahead_filters::lookahead_selector::SMatchOutput;
use crate::algorithms::compose::lookahead_filters::{
    LookAheadComposeFilter, PushLabelsComposeFilter, PushWeightsComposeFilter,
};
use crate::algorithms::compose::lookahead_matchers::label_lookahead_relabeler::LabelLookAheadRelabeler;
use crate::algorithms::compose::lookahead_matchers::{
    LabelLookAheadMatcher, LookaheadMatcher, MatcherFlagsTrait,
};
use crate::algorithms::compose::matchers::SortedMatcher;
use crate::algorithms::compose::matchers::{MatchType, Matcher, MatcherFlags};
use crate::algorithms::compose::MatcherFst;
use crate::algorithms::compose::{compose_with_config, ComposeConfig, LabelReachableData};
use crate::algorithms::compose::{ComposeFilterEnum, ComposeFst, ComposeFstOpOptions};
use crate::algorithms::{tr_compares::ilabel_compare, tr_sort};
use crate::fst_impls::VectorFst;
use crate::fst_traits::{CoreFst, SerializableFst};
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
) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
{
    // println!("Skipping simple compose for debugging");
    let mut config = ComposeConfig::default();
    config.connect = false;
    config.compose_filter = filter;

    let fst_res_static: VectorFst<_> = compose_with_config(
        Arc::new(fst_raw.clone()),
        Arc::new(compose_test_data.fst_2.clone()),
        config,
    )?;

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

#[derive(Debug, Clone, PartialOrd, PartialEq)]
struct DefaultLabelLookAheadMatcherFlags {}

impl MatcherFlagsTrait for DefaultLabelLookAheadMatcherFlags {
    fn flags() -> MatcherFlags {
        MatcherFlags::OUTPUT_LOOKAHEAD_MATCHER
            | MatcherFlags::LOOKAHEAD_WEIGHT
            | MatcherFlags::LOOKAHEAD_PREFIX
            | MatcherFlags::LOOKAHEAD_EPSILONS
            | MatcherFlags::LOOKAHEAD_NON_EPSILON_PREFIX
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct OLabelLookAheadFlags {}

impl MatcherFlagsTrait for OLabelLookAheadFlags {
    fn flags() -> MatcherFlags {
        MatcherFlags::OLABEL_LOOKAHEAD_FLAGS
    }
}

#[allow(unused)]
fn do_test_compose_lookahead<W>(
    fst_raw: &VectorFst<W>,
    compose_test_data: &ComposeTestData<VectorFst<W>>,
) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
{
    type TLaFst<F> = MatcherFst<
        F,
        LabelLookAheadMatcher<
            <F as CoreFst>::W,
            SortedMatcher<F>,
            DefaultLabelLookAheadMatcherFlags,
        >,
        LabelReachableData,
    >;

    type TMatcher1<F> = LabelLookAheadMatcher<
        <F as CoreFst>::W,
        SortedMatcher<F>,
        DefaultLabelLookAheadMatcherFlags,
    >;
    type TMatcher2<F> = SortedMatcher<F>;

    type TSeqFilter<S, F1, F2> = AltSequenceComposeFilter<S, TMatcher1<F1>, TMatcher2<F2>>;
    type TLookFilter<S, F1, F2> = LookAheadComposeFilter<S, TSeqFilter<S, F1, F2>, SMatchOutput>;
    type TPushWeightsFilter<S, F1, F2> =
        PushWeightsComposeFilter<S, TLookFilter<S, F1, F2>, SMatchOutput>;
    type TPushLabelsFilter<S, F1, F2> =
        PushLabelsComposeFilter<S, TPushWeightsFilter<S, F1, F2>, SMatchOutput>;

    type TComposeFilter<S, F1, F2> = TPushLabelsFilter<S, F1, F2>;

    let fst1: VectorFst<_> = fst_raw.clone().into();
    let mut fst2: VectorFst<_> = compose_test_data.fst_2.clone();

    let graph1look = Arc::new(TLaFst::new(fst1)?);

    LabelLookAheadRelabeler::relabel(&mut fst2, graph1look.addon(), true)?;

    tr_sort(&mut fst2, ilabel_compare);
    let fst2 = Arc::new(fst2);

    let matcher1 = TMatcher1::new_with_data(
        Arc::clone(&graph1look),
        MatchType::MatchOutput,
        graph1look.data(MatchType::MatchOutput).cloned(),
    )?;
    let matcher2 = TMatcher2::new(Arc::clone(&fst2), MatchType::MatchInput)?;

    let compose_filter = TPushLabelsFilter::new_2(
        Arc::clone(&graph1look),
        Arc::clone(&fst2),
        Arc::new(RefCell::new(matcher1)),
        Arc::new(RefCell::new(matcher2)),
    )?;

    // let compose_filter = TComposeFilter::new(
    //     &graph1look,
    //     &fst2,
    //     Arc::new(RefCell::new(matcher1)),
    //     Arc::new(RefCell::new(matcher2)),
    // )?;

    let compose_options = ComposeFstOpOptions::<_, _, TComposeFilter<_, _, _>, _>::new(
        compose_filter.matcher1(),
        compose_filter.matcher2(),
        compose_filter,
        None,
    );

    let dyn_fst = ComposeFst::new_with_options(graph1look, fst2, compose_options)?;
    let static_fst: VectorFst<_> = dyn_fst.compute()?;

    assert_eq!(
        static_fst,
        compose_test_data.result,
        "{}",
        error_message_fst!(
            compose_test_data.result,
            static_fst,
            format!("Compose failed : filter_name = lookahead")
        )
    );

    Ok(())
}

pub fn test_compose<W>(test_data: &FstTestData<VectorFst<W>>) -> Result<()>
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
