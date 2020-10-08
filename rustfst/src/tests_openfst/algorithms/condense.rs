use std::fmt::Display;
use std::marker::PhantomData;

use anyhow::Result;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::algorithms::condense;
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::{SerializableSemiring, WeightQuantize};
use crate::tests_openfst::utils::test_eq_fst;
use crate::tests_openfst::FstTestData;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct CondenseOperationResult {
    sccs: Vec<String>,
    result_path: String,
}

pub struct CondenseTestData<W, F>
where
    F: SerializableFst<W>,
    W: SerializableSemiring,
{
    pub sccs: Vec<i32>,
    pub result: F,
    w: PhantomData<W>,
}

impl CondenseOperationResult {
    pub fn parse<W, F, P>(&self, dir_path: P) -> CondenseTestData<W, F>
    where
        F: SerializableFst<W>,
        W: SerializableSemiring,
        P: AsRef<Path>,
    {
        CondenseTestData {
            result: F::read(dir_path.as_ref().join(&self.result_path)).unwrap(),
            sccs: self.sccs.iter().map(|e| e.parse().unwrap()).collect_vec(),
            w: PhantomData,
        }
    }
}

pub fn test_condense<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: MutableFst<W> + Display + SerializableFst<W>,
    W: SerializableSemiring + WeightQuantize,
{
    // Connect
    let fst_in = test_data.raw.clone();
    let (sccs, fst_condensed): (_, F) = condense(&fst_in)?;

    assert_eq!(sccs, test_data.condense.sccs);

    test_eq_fst(&test_data.condense.result, &fst_condensed, "Condense");

    Ok(())
}
