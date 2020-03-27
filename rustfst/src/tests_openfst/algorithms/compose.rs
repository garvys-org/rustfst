use failure::Fallible;
use serde_derive::{Deserialize, Serialize};

use crate::algorithms::compose_filters::AltSequenceComposeFilter;
use crate::algorithms::lookahead_filters::lookahead_selector::SMatchOutput;
use crate::algorithms::lookahead_filters::{LookAheadComposeFilter, PushWeightsComposeFilter};
use crate::algorithms::lookahead_matchers::label_lookahead_relabeler::LabelLookAheadRelabeler;
use crate::algorithms::lookahead_matchers::label_reachable::LabelReachableData;
use crate::algorithms::lookahead_matchers::matcher_fst::MatcherFst;
use crate::algorithms::lookahead_matchers::{LabelLookAheadMatcher, MatcherFlagsTrait};
use crate::algorithms::matchers::SortedMatcher;
use crate::algorithms::matchers::{MatchType, Matcher, MatcherFlags};
use crate::algorithms::{arc_compares::ilabel_compare, arc_sort};
use crate::algorithms::{
    compose_with_config, ComposeConfig, ComposeFilterEnum, ComposeFst, ComposeFstImplOptions,
};
use crate::fst_impls::{ConstFst, VectorFst};
use crate::fst_traits::{CoreFst, Fst, SerializableFst};
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

#[derive(Debug, Clone, PartialOrd, PartialEq)]
struct DefaultLabelLookAheadMatcherFlags {}

impl MatcherFlagsTrait for DefaultLabelLookAheadMatcherFlags {
    fn flags() -> MatcherFlags {
        MatcherFlags::LOOKAHEAD_EPSILONS
            | MatcherFlags::LOOKAHEAD_WEIGHT
            | MatcherFlags::LOOKAHEAD_PREFIX
            | MatcherFlags::LOOKAHEAD_NON_EPSILON_PREFIX
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
struct OLabelLookAheadFlags {}

impl MatcherFlagsTrait for OLabelLookAheadFlags {
    fn flags() -> MatcherFlags {
        MatcherFlags::OLABEL_LOOKAHEAD_FLAGS
    }
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
    type LA_FST<'fst, F> = MatcherFst<
        F,
        LabelLookAheadMatcher<
            'fst,
            <F as CoreFst>::W,
            SortedMatcher<'fst, F>,
            OLabelLookAheadFlags,
        >,
        LabelReachableData,
    >;

    type MATCHER1<'a, F> = LabelLookAheadMatcher<
        'a,
        <F as CoreFst>::W,
        SortedMatcher<'a, F>,
        DefaultLabelLookAheadMatcherFlags,
    >;
    type MATCHER2<'a, F> = SortedMatcher<'a, F>;

    type SEQFILTER<'fst1, 'fst2, S> =
        AltSequenceComposeFilter<'fst1, 'fst2, S, MATCHER1<'fst1, S>, MATCHER2<'fst2, S>>;
    type LOOKFILTER<'fst1, 'fst2, S> =
        LookAheadComposeFilter<'fst1, 'fst2, S, SEQFILTER<'fst1, 'fst2, S>, SMatchOutput>;
    type PUSHWEIGHTSFILTER<'fst1, 'fst2, S> =
        PushWeightsComposeFilter<'fst1, 'fst2, S, SEQFILTER<'fst1, 'fst2, S>, SMatchOutput>;

    type COMPOSEFILTER<'fst1, 'fst2, S> = LOOKFILTER<'fst1, 'fst2, S>;

    let fst1: VectorFst<_> = fst_raw.clone().into();
    let mut fst2: VectorFst<_> = compose_test_data.fst_2.clone();

    let graph1look = LA_FST::new(fst1)?;

    LabelLookAheadRelabeler::relabel(&mut fst2, graph1look.addon(), true)?;

    arc_sort(&mut fst2, ilabel_compare);

    // FIXME: Move to init matcher
    let matcher1 = MATCHER1::new(&graph1look, MatchType::MatchOutput)?;
    let matcher2 = MATCHER2::new(&fst2, MatchType::MatchInput)?;

    // let compose_options = ComposeFstImplOptions::<MATCHER1<_>, MATCHER2<_>, LOOKFILTER<_>, _>::new(matcher1, matcher2, None, None);
    //
    // let dyn_fst = ComposeFst::<_, _>::new_with_options(&graph1look, &fst2, compose_options)?;
    // let static_fst: VectorFst<_> = dyn_fst.compute()?;

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
