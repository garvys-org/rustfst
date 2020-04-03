use std::fmt::Display;

use failure::Fallible;
use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};

use crate::algorithms::compose::lookahead_matchers::LabelLookAheadMatcher;
use crate::algorithms::compose::matchers::SortedMatcher;
use crate::algorithms::compose::{
    IntInterval, LabelReachable, LabelReachableData, MatcherFst, StateReachable,
};
use crate::algorithms::condense;
use crate::fst_impls::VectorFst;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{CoreFst, MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::tests_openfst::algorithms::compose::OLabelLookAheadFlags;
use crate::tests_openfst::FstTestData;
use crate::NO_LABEL;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReachabilityTestResult {
    state: usize,
    label: usize,
    label_relabeled: usize,
    reach_final: bool,
    reachable: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LabelReachableOperationResult {
    result: Vec<ReachabilityTestResult>,
    reach_input: bool,
    final_label: i32,
    interval_sets: Vec<Vec<IntInterval>>,
}

pub struct LabelReachableTestData {
    pub result: Vec<ReachabilityTestResult>,
    pub reach_input: bool,
    pub final_label: usize,
    pub interval_sets: Vec<Vec<IntInterval>>,
}

impl LabelReachableOperationResult {
    pub fn parse(&self) -> LabelReachableTestData {
        LabelReachableTestData {
            result: self.result.clone(),
            reach_input: self.reach_input,
            final_label: if self.final_label == -1 {
                NO_LABEL
            } else {
                self.final_label as usize
            },
            interval_sets: self.interval_sets.clone(),
        }
    }
}

pub fn test_label_reachable<F>(test_data: &FstTestData<F>) -> Fallible<()>
where
    F: MutableFst + Display + SerializableFst,
    F::W: SerializableSemiring + 'static,
{
    type TLaFst<F> = MatcherFst<
        F,
        LabelLookAheadMatcher<<F as CoreFst>::W, SortedMatcher<F>, OLabelLookAheadFlags>,
        LabelReachableData,
    >;

    let fst = TLaFst::new(test_data.raw.clone())?;

    for label_reachable_test_data in &test_data.label_reachable {
        let reachable = LabelReachable::new(&fst, label_reachable_test_data.reach_input)?;

        let reachable_data = reachable.data().borrow();
        assert_eq!(
            reachable_data.final_label(),
            label_reachable_test_data.final_label
        );

        for i in 0..label_reachable_test_data.interval_sets.len() {
            let interval_set = reachable_data.interval_set(i)?;
            assert_eq!(
                interval_set.intervals.intervals,
                label_reachable_test_data.interval_sets[i]
            );
        }

        drop(reachable_data);

        for data in &label_reachable_test_data.result {
            let current_state = data.state;
            let label = data.label;
            let label_relabeled = reachable.relabel(label);
            assert_eq!(label_relabeled, data.label_relabeled);
            let res = reachable.reach_label(current_state, label_relabeled)?;
            assert_eq!(res, data.reachable);
            assert_eq!(reachable.reach_final(current_state)?, data.reach_final);
        }
    }
    Ok(())
}
