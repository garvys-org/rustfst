use std::fmt::Display;
use std::marker::PhantomData;

use anyhow::{format_err, Result};
use serde::{Deserialize, Serialize};

use crate::algorithms::minimize;
use crate::fst_traits::{AllocableFst, MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::semirings::WeaklyDivisibleSemiring;
use crate::semirings::WeightQuantize;
use crate::tests_openfst::macros::test_eq_fst;
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct MinimizeOperationResult {
    allow_nondet: bool,
    result: String,
}

pub struct MinimizeTestData<W, F>
where
    F: SerializableFst<W>,
    W: SerializableSemiring,
{
    allow_nondet: bool,
    result: Result<F>,
    w: PhantomData<W>,
}

impl MinimizeOperationResult {
    pub fn parse<W, F>(&self) -> MinimizeTestData<W, F>
    where
        F: SerializableFst<W>,
        W: SerializableSemiring,
    {
        MinimizeTestData {
            allow_nondet: self.allow_nondet,
            result: match self.result.as_str() {
                "error" => Err(format_err!("lol")),
                _ => F::from_text_string(self.result.as_str()),
            },
            w: PhantomData,
        }
    }
}

pub fn test_minimize<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + AllocableFst<W> + Display,
    W: SerializableSemiring + WeaklyDivisibleSemiring + WeightQuantize,
{
    for minimize_data in &test_data.minimize {
        //        println!("Minimize : allow_nondet = {}", minimize_data.allow_nondet);
        let mut fst_raw = test_data.raw.clone();
        let fst_res: Result<F> =
            minimize(&mut fst_raw, minimize_data.allow_nondet).map(|_| fst_raw);

        match (&minimize_data.result, fst_res) {
            (Ok(fst_expected), Ok(ref fst_minimized)) => {
                test_eq_fst(
                    fst_expected,
                    fst_minimized,
                    format!(
                        "Minimize fail for allow_nondet = {:?} ",
                        minimize_data.allow_nondet
                    ),
                );
            }
            (Ok(_fst_expected), Err(_)) => panic!(
                "Minimize fail for allow_nondet {:?}. Got Err. Expected Ok",
                minimize_data.allow_nondet
            ),
            (Err(_), Ok(_fst_minimized)) => panic!(
                "Minimize fail for allow_nondet {:?}. Got Ok. Expected Err, \n{}",
                minimize_data.allow_nondet, _fst_minimized
            ),
            (Err(_), Err(_)) => {
                // Ok
            }
        };
    }
    Ok(())
}
