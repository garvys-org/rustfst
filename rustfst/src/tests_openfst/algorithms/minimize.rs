use std::fmt::Display;
use std::marker::PhantomData;

use anyhow::{format_err, Result};
use serde::{Deserialize, Serialize};

use crate::algorithms::{minimize_with_config, MinimizeConfig};
use crate::fst_traits::{AllocableFst, MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::semirings::WeaklyDivisibleSemiring;
use crate::semirings::WeightQuantize;
use crate::tests_openfst::utils::test_isomorphic_fst;
use crate::tests_openfst::FstTestData;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct MinimizeOperationResult {
    delta: f32,
    allow_nondet: bool,
    result_path: String,
}

pub struct MinimizeTestData<W, F>
where
    F: SerializableFst<W>,
    W: SerializableSemiring,
{
    delta: f32,
    allow_nondet: bool,
    result: Result<F>,
    w: PhantomData<W>,
}

impl MinimizeOperationResult {
    pub fn parse<W, F, P>(&self, dir_path: P) -> MinimizeTestData<W, F>
    where
        F: SerializableFst<W>,
        W: SerializableSemiring,
        P: AsRef<Path>,
    {
        MinimizeTestData {
            delta: self.delta,
            allow_nondet: self.allow_nondet,
            result: match self.result_path.as_str() {
                "error" => Err(format_err!("lol")),
                _ => F::read(dir_path.as_ref().join(&self.result_path)),
            },
            w: PhantomData,
        }
    }
}

pub fn test_minimize<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + AllocableFst<W> + Display,
    W: SerializableSemiring + WeaklyDivisibleSemiring + WeightQuantize,
    W::ReverseWeight: WeightQuantize,
{
    for minimize_data in &test_data.minimize {
        println!("Minimize : allow_nondet = {}", minimize_data.allow_nondet);
        let mut fst_raw = test_data.raw.clone();
        let fst_res: Result<F> = minimize_with_config(
            &mut fst_raw,
            MinimizeConfig::new(minimize_data.delta, minimize_data.allow_nondet),
        )
        .map(|_| fst_raw);

        match (&minimize_data.result, fst_res) {
            (Ok(fst_expected), Ok(ref fst_minimized)) => {
                test_isomorphic_fst(
                    fst_expected,
                    fst_minimized,
                    format!(
                        "Minimize fail for allow_nondet = {:?} ",
                        minimize_data.allow_nondet
                    ),
                );
            }
            (Ok(_fst_expected), Err(ref e)) => panic!(
                "Minimize fail for allow_nondet {:?}. Got Err {:?}. Expected Ok",
                minimize_data.allow_nondet, e
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
