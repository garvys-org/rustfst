use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::algorithms::compose::compose_filters::{
    AltSequenceComposeFilterBuilder, ComposeFilterBuilder,
};
use crate::algorithms::compose::lookahead_filters::lookahead_selector::SMatchOutput;
use crate::algorithms::compose::lookahead_filters::{
    LookAheadComposeFilterBuilder, PushLabelsComposeFilterBuilder, PushWeightsComposeFilterBuilder,
};
use crate::algorithms::compose::lookahead_matchers::{
    LabelLookAheadMatcher, LookaheadMatcher, MatcherFlagsTrait,
};
use crate::algorithms::compose::matchers::SortedMatcher;
use crate::algorithms::compose::matchers::{MatchType, Matcher, MatcherFlags};
use crate::algorithms::compose::MatcherFst;
use crate::algorithms::compose::{compose_with_config, ComposeConfig, LabelReachableData};
use crate::algorithms::compose::{ComposeFilterEnum, ComposeFst, ComposeFstOpOptions};
use crate::algorithms::lazy::SimpleHashMapCache;
use crate::algorithms::{tr_compares::ILabelCompare, tr_sort};
use crate::fst_impls::VectorFst;
use crate::fst_traits::{CoreFst, ExpandedFst, Fst, MutableFst, SerializableFst};
use crate::prelude::lazy::SerializableLazyFst;
use crate::semirings::{SerializableSemiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::utils::test_eq_fst;
use crate::tests_openfst::FstTestData;
use crate::{StateId, SymbolTable, Trs};
use std::cmp::max;

#[derive(Serialize, Deserialize, Debug)]
pub struct ComposeOperationResult {
    fst_2_path: String,
    result_path: String,
    filter_name: String,
}

pub struct ComposeTestData<W, F>
where
    F: SerializableFst<W>,
    W: SerializableSemiring,
{
    pub fst_2: F,
    pub result: F,
    pub filter_name: String,
    w: PhantomData<W>,
}

impl ComposeOperationResult {
    pub fn parse<W, F, P>(&self, dir_path: P) -> ComposeTestData<W, F>
    where
        F: SerializableFst<W>,
        W: SerializableSemiring,
        P: AsRef<Path>,
    {
        ComposeTestData {
            fst_2: F::read(dir_path.as_ref().join(&self.fst_2_path)).unwrap(),
            result: F::read(dir_path.as_ref().join(&self.result_path)).unwrap(),
            filter_name: self.filter_name.clone(),
            w: PhantomData,
        }
    }
}

fn do_test_compose<W>(
    fst_raw: &VectorFst<W>,
    compose_test_data: &ComposeTestData<W, VectorFst<W>>,
    filter: ComposeFilterEnum,
) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
{
    let mut config = ComposeConfig::default();
    config.connect = false;
    config.compose_filter = filter;

    let fst_res_static: VectorFst<_> = compose_with_config::<W, VectorFst<_>, VectorFst<_>, _, _, _>(
        Arc::new(fst_raw.clone()),
        Arc::new(compose_test_data.fst_2.clone()),
        config,
    )?;

    test_eq_fst(
        &compose_test_data.result,
        &fst_res_static,
        format!(
            "Compose failed : filter_name = {:?}",
            compose_test_data.filter_name
        ),
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

fn do_test_compose_lookahead<W>(
    fst_raw: &VectorFst<W>,
    compose_test_data: &ComposeTestData<W, VectorFst<W>>,
) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
{
    println!("Lookahead composition");
    type TLaFst<S, F> = MatcherFst<
        S,
        F,
        Arc<F>,
        LabelLookAheadMatcher<
            S,
            F,
            Arc<F>,
            SortedMatcher<S, F, Arc<F>>,
            DefaultLabelLookAheadMatcherFlags,
        >,
        LabelReachableData,
    >;

    type TMatcher1<S, F> = LabelLookAheadMatcher<
        S,
        F,
        Arc<F>,
        SortedMatcher<S, F, Arc<F>>,
        DefaultLabelLookAheadMatcherFlags,
    >;
    type TMatcher2<S, F> = SortedMatcher<S, F, Arc<F>>;

    type TSeqFilter<S, F1, F2> = AltSequenceComposeFilterBuilder<
        S,
        F1,
        F2,
        Arc<F1>,
        Arc<F2>,
        TMatcher1<S, F1>,
        TMatcher2<S, F2>,
    >;
    type TLookFilter<S, F1, F2> = LookAheadComposeFilterBuilder<
        S,
        F1,
        F2,
        Arc<F1>,
        Arc<F2>,
        TMatcher1<S, F1>,
        TMatcher2<S, F2>,
        TSeqFilter<S, F1, F2>,
        SMatchOutput,
    >;
    type TPushWeightsFilter<S, F1, F2> = PushWeightsComposeFilterBuilder<
        S,
        F1,
        F2,
        Arc<F1>,
        Arc<F2>,
        TMatcher1<S, F1>,
        TMatcher2<S, F2>,
        TLookFilter<S, F1, F2>,
        SMatchOutput,
    >;
    type TPushLabelsFilter<S, F1, F2> = PushLabelsComposeFilterBuilder<
        S,
        F1,
        F2,
        Arc<F1>,
        Arc<F2>,
        TMatcher1<S, F1>,
        TMatcher2<S, F2>,
        TPushWeightsFilter<S, F1, F2>,
        SMatchOutput,
    >;

    type TComposeFilter<S, F1, F2> = TPushLabelsFilter<S, F1, F2>;

    let fst1: VectorFst<_> = fst_raw.clone().into();
    let mut fst2: VectorFst<_> = compose_test_data.fst_2.clone();

    let graph1look = Arc::new(TLaFst::new_with_relabeling(fst1, &mut fst2, true)?);

    // LabelLookAheadRelabeler::relabel(&mut fst2, graph1look.addon(), true)?;

    tr_sort(&mut fst2, ILabelCompare {});

    let fst2 = Arc::new(fst2);

    let matcher1 = TMatcher1::new_with_data(
        Arc::clone(&graph1look),
        MatchType::MatchOutput,
        graph1look.data(MatchType::MatchOutput).cloned(),
    )?;

    let matcher2 = TMatcher2::new(Arc::clone(&fst2), MatchType::MatchInput)?;

    let compose_filter = TComposeFilter::new(
        Arc::clone(&graph1look),
        Arc::clone(&fst2),
        Some(matcher1),
        Some(matcher2),
    )?;

    // let compose_filter = TComposeFilter::new(
    //     &graph1look,
    //     &fst2,
    //     Arc::new(RefCell::new(matcher1)),
    //     Arc::new(RefCell::new(matcher2)),
    // )?;

    let compose_options = ComposeFstOpOptions::<_, _, TComposeFilter<_, _, _>, _>::new(
        // compose_filter.matcher1(),
        None,
        // compose_filter.matcher2(),
        None,
        compose_filter,
        None,
    );

    let dyn_fst = ComposeFst::<_, _, _, _, _, _, _, _, SimpleHashMapCache<_>>::new_with_options(
        graph1look,
        fst2,
        compose_options,
    )?;

    // Check clonability
    fn is_clone<T: Clone>(_v: &T) {}
    is_clone(&dyn_fst);

    let static_fst: VectorFst<_> = dyn_fst.compute()?;

    test_eq_fst(
        &compose_test_data.result,
        &static_fst,
        format!("Compose failed : filter_name = lookahead"),
    );

    Ok(())
}

fn do_test_compose_lookahead_lazy_right<W>(
    fst_raw: &VectorFst<W>,
    compose_test_data: &ComposeTestData<W, VectorFst<W>>,
) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
{
    println!("Lookahead composition lazy right");
    type TLaFst<S, F> = MatcherFst<
        S,
        F,
        Arc<F>,
        LabelLookAheadMatcher<
            S,
            F,
            Arc<F>,
            SortedMatcher<S, F, Arc<F>>,
            DefaultLabelLookAheadMatcherFlags,
        >,
        LabelReachableData,
    >;

    type TMatcher1<S, F> = LabelLookAheadMatcher<
        S,
        F,
        Arc<F>,
        SortedMatcher<S, F, Arc<F>>,
        DefaultLabelLookAheadMatcherFlags,
    >;
    type TMatcher2<S, F> = SortedMatcher<S, F, Arc<F>>;

    type TSeqFilter<S, F1, F2> = AltSequenceComposeFilterBuilder<
        S,
        F1,
        F2,
        Arc<F1>,
        Arc<F2>,
        TMatcher1<S, F1>,
        TMatcher2<S, F2>,
    >;
    type TLookFilter<S, F1, F2> = LookAheadComposeFilterBuilder<
        S,
        F1,
        F2,
        Arc<F1>,
        Arc<F2>,
        TMatcher1<S, F1>,
        TMatcher2<S, F2>,
        TSeqFilter<S, F1, F2>,
        SMatchOutput,
    >;
    type TPushWeightsFilter<S, F1, F2> = PushWeightsComposeFilterBuilder<
        S,
        F1,
        F2,
        Arc<F1>,
        Arc<F2>,
        TMatcher1<S, F1>,
        TMatcher2<S, F2>,
        TLookFilter<S, F1, F2>,
        SMatchOutput,
    >;
    type TPushLabelsFilter<S, F1, F2> = PushLabelsComposeFilterBuilder<
        S,
        F1,
        F2,
        Arc<F1>,
        Arc<F2>,
        TMatcher1<S, F1>,
        TMatcher2<S, F2>,
        TPushWeightsFilter<S, F1, F2>,
        SMatchOutput,
    >;

    type TComposeFilter<S, F1, F2> = TPushLabelsFilter<S, F1, F2>;

    let fst1: VectorFst<_> = fst_raw.clone().into();
    let mut fst2: VectorFst<_> = compose_test_data.fst_2.clone();

    let mut symt = SymbolTable::new();
    let mut max_ilabel = 0;
    for s in 0..fst2.num_states() {
        let trs = fst2.get_trs(s as StateId)?;
        for tr in trs.trs() {
            max_ilabel = max(max_ilabel, tr.ilabel);
        }
    }
    for i in 1..=max_ilabel {
        symt.add_symbol(i.to_string());
    }
    fst2.set_input_symbols(Arc::new(symt));

    let (graph1look, lazy_fst2) =
        TLaFst::new_with_relabeling_lazy_right(fst1, Arc::new(fst2), true)?;
    let graph1look = Arc::new(graph1look);
    let fst2 = Arc::new(lazy_fst2);

    // LabelLookAheadRelabeler::relabel(&mut fst2, graph1look.addon(), true)?;

    // tr_sort(&mut fst2, ILabelCompare {});

    // let fst2 = Arc::new(fst2);

    let matcher1 = TMatcher1::new_with_data(
        Arc::clone(&graph1look),
        MatchType::MatchOutput,
        graph1look.data(MatchType::MatchOutput).cloned(),
    )?;

    let matcher2 = TMatcher2::new(Arc::clone(&fst2), MatchType::MatchInput)?;

    let compose_filter = TComposeFilter::new(
        Arc::clone(&graph1look),
        Arc::clone(&fst2),
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

    let dyn_fst = ComposeFst::<_, _, _, _, _, _, _, _, SimpleHashMapCache<_>>::new_with_options(
        graph1look,
        fst2,
        compose_options,
    )?;

    // Check clonability
    fn is_clone<T: Clone>(_v: &T) {}
    is_clone(&dyn_fst);

    fn is_serializable<T: SerializableLazyFst>(_v: &T) {}
    is_serializable(&dyn_fst);

    let mut static_fst: VectorFst<_> = dyn_fst.compute()?;
    // TODO: This is a hotfix that should be removed.
    static_fst.compute_and_update_properties_all()?;

    test_eq_fst(
        &compose_test_data.result,
        &static_fst,
        format!("Compose failed : filter_name = lookahead_lazy_right"),
    );

    Ok(())
}

pub fn test_compose<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
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
            "lookahead" => {
                do_test_compose_lookahead(&test_data.raw, compose_test_data)?;
                do_test_compose_lookahead_lazy_right(&test_data.raw, compose_test_data)?;
            }
            _ => panic!("Not supported : {}", &compose_test_data.filter_name),
        }
    }
    Ok(())
}
