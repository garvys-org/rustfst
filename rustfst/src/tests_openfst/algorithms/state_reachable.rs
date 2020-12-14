use std::fmt::Display;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::algorithms::compose::StateReachable;
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::tests_openfst::FstTestData;
use crate::StateId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReachabilityTestResult {
    state: StateId,
    final_state: StateId,
    reachable: bool,
    error: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StateReachableOperationResult {
    result: Vec<ReachabilityTestResult>,
    error: bool,
}

pub struct StateReachableTestData {
    pub result: Vec<ReachabilityTestResult>,
    pub error: bool,
}

impl StateReachableOperationResult {
    pub fn parse(&self) -> StateReachableTestData {
        StateReachableTestData {
            result: self.result.clone(),
            error: self.error,
        }
    }
}

pub fn test_state_reachable<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: MutableFst<W> + Display + SerializableFst<W>,
    W: SerializableSemiring,
{
    let state_reachable_test_data = &test_data.state_reachable;
    let reachable = StateReachable::new(&test_data.raw);
    if state_reachable_test_data.error {
        assert!(reachable.is_err());
        return Ok(());
    }
    let reachable = reachable?;
    for reachability_test in &test_data.state_reachable.result {
        let res = reachable.reach(reachability_test.state, reachability_test.final_state);
        if reachability_test.error {
            assert!(res.is_err());
            continue;
        }
        let res = res?;
        assert_eq!(
            res, reachability_test.reachable,
            "State Reachable test failing : state = {} final_state = {}",
            reachability_test.state, reachability_test.final_state
        );
    }
    Ok(())
}
