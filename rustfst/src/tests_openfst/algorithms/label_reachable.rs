use std::fmt::Display;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::algorithms::compose::lookahead_matchers::LabelLookAheadMatcher;
use crate::algorithms::compose::matchers::SortedMatcher;
use crate::algorithms::compose::{IntInterval, LabelReachable, LabelReachableData, MatcherFst};
use crate::fst_traits::{CoreFst, MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::tests_openfst::algorithms::compose::OLabelLookAheadFlags;
use crate::tests_openfst::FstTestData;
use crate::NO_LABEL;
use std::sync::Arc;

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

pub fn test_label_reachable<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: MutableFst<W> + Display + SerializableFst<W>,
    W: SerializableSemiring,
{
    type TLaFst<S, F> = MatcherFst<
        S,
        F,
        LabelLookAheadMatcher<S, SortedMatcher<S, F>, OLabelLookAheadFlags>,
        LabelReachableData,
    >;

    let fst = TLaFst::new(test_data.raw.clone())?;

    for label_reachable_test_data in &test_data.label_reachable {
        let mut reachable_data =
            LabelReachable::compute_data(&fst, label_reachable_test_data.reach_input)?.0;

        // Mutable operations done at the beginning
        for data in &label_reachable_test_data.result {
            let label = data.label;
            let label_relabeled = reachable_data.relabel(label);
            assert_eq!(label_relabeled, data.label_relabeled);
        }

        let reachable = LabelReachable::new_from_data(Arc::new(reachable_data));

        let reachable_data = reachable.data();

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

        for data in &label_reachable_test_data.result {
            let current_state = data.state;
            let label = data.label;
            let label_relabeled = reachable_data.label2index()[&label];
            assert_eq!(label_relabeled, data.label_relabeled);
            let res = reachable.reach_label(current_state, label_relabeled)?;
            assert_eq!(res, data.reachable);
            assert_eq!(reachable.reach_final(current_state)?, data.reach_final);
        }
    }
    Ok(())
}
