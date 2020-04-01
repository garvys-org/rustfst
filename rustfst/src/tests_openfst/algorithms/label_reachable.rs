use std::fmt::Display;

use failure::Fallible;
use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};

use crate::algorithms::compose::{StateReachable, LabelReachable};
use crate::algorithms::condense;
use crate::fst_impls::VectorFst;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReachabilityTestResult {
    state: usize,
    label: usize,
    label_relabeled: usize,
    reach_final: bool,
    reachable: bool
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LabelReachableOperationResult {
    result: Vec<ReachabilityTestResult>,
    reach_input: bool
}

pub struct LabelReachableTestData {
    pub result: Vec<ReachabilityTestResult>,
    pub reach_input: bool
}

impl LabelReachableOperationResult {
    pub fn parse(&self) -> LabelReachableTestData {
        LabelReachableTestData {
            result: self.result.clone(),
            reach_input: self.reach_input,
        }
    }
}

pub fn test_label_reachable<F>(test_data: &FstTestData<F>) -> Fallible<()>
    where
        F: MutableFst + Display + SerializableFst,
        F::W: SerializableSemiring + 'static,
{
    for label_reachable_test_data in &test_data.label_reachable {
        let reachable = LabelReachable::new(&test_data.raw, label_reachable_test_data.reach_input)?;

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
