use std::fmt::Display;

use anyhow::Result;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::algorithms::condense;
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct CondenseOperationResult {
    sccs: Vec<String>,
    result: String,
}

pub struct CondenseTestData<F>
where
    F: SerializableFst,
    F::W: SerializableSemiring,
{
    pub sccs: Vec<i32>,
    pub result: F,
}

impl CondenseOperationResult {
    pub fn parse<F>(&self) -> CondenseTestData<F>
    where
        F: SerializableFst,
        F::W: SerializableSemiring,
    {
        CondenseTestData {
            result: F::from_text_string(self.result.as_str()).unwrap(),
            sccs: self.sccs.iter().map(|e| e.parse().unwrap()).collect_vec(),
        }
    }
}

pub fn test_condense<F>(test_data: &FstTestData<F>) -> Result<()>
where
    F: MutableFst + Display + SerializableFst,
    F::W: SerializableSemiring,
{
    // Connect
    let fst_in = test_data.raw.clone();
    let (sccs, fst_condensed): (_, F) = condense(&fst_in)?;

    assert_eq!(sccs, test_data.condense.sccs);

    assert_eq!(
        test_data.condense.result,
        fst_condensed,
        "{}",
        error_message_fst!(test_data.condense.result, fst_condensed, "Condense")
    );
    Ok(())
}
