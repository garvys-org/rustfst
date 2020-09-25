use std::sync::Arc;

use anyhow::Result;

use rustfst::algorithms::compose::compose_filters::{
    AltSequenceComposeFilterBuilder, ComposeFilterBuilder,
};
use rustfst::algorithms::compose::lookahead_filters::{
    LookAheadComposeFilterBuilder, PushLabelsComposeFilterBuilder, PushWeightsComposeFilterBuilder,
    SMatchOutput,
};
use rustfst::algorithms::compose::lookahead_matchers::{
    LabelLookAheadMatcher, LookaheadMatcher, MatcherFlagsTrait,
};
use rustfst::algorithms::compose::matchers::{MatchType, Matcher, MatcherFlags, SortedMatcher};
use rustfst::algorithms::compose::{
    compose, ComposeFst, ComposeFstOpOptions, LabelReachableData, MatcherFst,
};
use rustfst::algorithms::lazy::SimpleVecCache;
use rustfst::algorithms::tr_compares::ILabelCompare;
use rustfst::algorithms::tr_sort;
use rustfst::fst_impls::VectorFst;
use rustfst::semirings::TropicalWeight;

use crate::binary_fst_algorithm::BinaryFstAlgorithm;

#[derive(Debug, Clone, Copy)]
pub enum ComposeType {
    Default,
    LookAhead,
}

pub struct ComposeAlgorithm {
    path_in_1: String,
    path_in_2: String,
    path_out: String,
    compose_type: ComposeType,
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

impl BinaryFstAlgorithm for ComposeAlgorithm {
    fn get_path_in_1(&self) -> &str {
        &self.path_in_1
    }

    fn get_path_in_2(&self) -> &str {
        &self.path_in_2
    }

    fn get_path_out(&self) -> &str {
        &self.path_out
    }

    fn get_algorithm_name(&self) -> String {
        "compose".to_string()
    }

    fn run_algorithm(
        &self,
        fst_1: VectorFst<TropicalWeight>,
        mut fst_2: VectorFst<TropicalWeight>,
    ) -> Result<VectorFst<TropicalWeight>> {
        match self.compose_type {
            ComposeType::Default => compose(Arc::new(fst_1), Arc::new(fst_2)),
            ComposeType::LookAhead => {
                type TLaFst<S, F> = MatcherFst<
                    S,
                    F,
                    LabelLookAheadMatcher<
                        S,
                        SortedMatcher<S, F>,
                        DefaultLabelLookAheadMatcherFlags,
                    >,
                    LabelReachableData,
                >;

                type TMatcher1<S, F> = LabelLookAheadMatcher<
                    S,
                    SortedMatcher<S, F>,
                    DefaultLabelLookAheadMatcherFlags,
                >;
                type TMatcher2<S, F> = SortedMatcher<S, F>;

                type TSeqFilter<S, F1, F2> =
                    AltSequenceComposeFilterBuilder<S, TMatcher1<S, F1>, TMatcher2<S, F2>>;
                type TLookFilter<S, F1, F2> =
                    LookAheadComposeFilterBuilder<S, TSeqFilter<S, F1, F2>, SMatchOutput>;
                type TPushWeightsFilter<S, F1, F2> =
                    PushWeightsComposeFilterBuilder<S, TLookFilter<S, F1, F2>, SMatchOutput>;
                type TPushLabelsFilter<S, F1, F2> =
                    PushLabelsComposeFilterBuilder<S, TPushWeightsFilter<S, F1, F2>, SMatchOutput>;

                type TComposeFilter<S, F1, F2> = TPushLabelsFilter<S, F1, F2>;

                let graph1look = Arc::new(TLaFst::new_with_relabeling(fst_1, &mut fst_2, true)?);

                // LabelLookAheadRelabeler::relabel(&mut fst2, graph1look.addon(), true)?;

                tr_sort(&mut fst_2, ILabelCompare {});

                let fst_2 = Arc::new(fst_2);

                let matcher1 = TMatcher1::new_with_data(
                    Arc::clone(&graph1look),
                    MatchType::MatchOutput,
                    graph1look.data(MatchType::MatchOutput).cloned(),
                )?;

                let matcher2 = TMatcher2::new(Arc::clone(&fst_2), MatchType::MatchInput)?;

                let compose_filter = TComposeFilter::new(
                    Arc::clone(&graph1look),
                    Arc::clone(&fst_2),
                    Some(matcher1),
                    Some(matcher2),
                )?;

                let compose_options = ComposeFstOpOptions::<_, _, TComposeFilter<_, _, _>, _>::new(
                    // compose_filter.matcher1(),
                    None,
                    // compose_filter.matcher2(),
                    None,
                    compose_filter,
                    None,
                );

                let dyn_fst = ComposeFst::<_, _, SimpleVecCache<_>>::new_with_options(
                    graph1look,
                    fst_2,
                    compose_options,
                )?;

                Ok(dyn_fst.compute_2())
            }
        }
    }
}

impl ComposeAlgorithm {
    pub fn new(path_in_1: &str, path_in_2: &str, path_out: &str, compose_type: &str) -> Self {
        let compose_type = match compose_type {
            "default" => ComposeType::Default,
            "lookahead" => ComposeType::LookAhead,
            _ => panic!("Unexpected compose_type : {}", compose_type),
        };
        Self {
            path_in_1: path_in_1.to_string(),
            path_in_2: path_in_2.to_string(),
            path_out: path_out.to_string(),
            compose_type,
        }
    }
}
