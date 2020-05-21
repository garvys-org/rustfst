use std::fmt::Display;
use std::marker::PhantomData;

use anyhow::{format_err, Result};
use serde::{Deserialize, Serialize};

use crate::algorithms::{isomorphic, shortest_path};
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::WeaklyDivisibleSemiring;
use crate::semirings::WeightQuantize;
use crate::semirings::{Semiring, SerializableSemiring};
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct ShorestPathOperationResult {
    unique: bool,
    nshortest: usize,
    result: String,
}

pub struct ShortestPathTestData<W, F>
where
    F: SerializableFst<W>,
    W: SerializableSemiring,
{
    unique: bool,
    nshortest: usize,
    result: Result<F>,
    w: PhantomData<W>,
}

impl ShorestPathOperationResult {
    pub fn parse<W, F>(&self) -> ShortestPathTestData<W, F>
    where
        F: SerializableFst<W>,
        W: SerializableSemiring,
    {
        ShortestPathTestData {
            unique: self.unique,
            nshortest: self.nshortest,
            result: match self.result.as_str() {
                "error" => Err(format_err!("lol")),
                _ => F::from_text_string(self.result.as_str()),
            },
            w: PhantomData,
        }
    }
}

pub fn test_shortest_path<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring + WeaklyDivisibleSemiring + WeightQuantize,
    <W as Semiring>::ReverseWeight: WeaklyDivisibleSemiring + WeightQuantize,
    W: Into<<W as Semiring>::ReverseWeight> + From<<W as Semiring>::ReverseWeight>,
{
    for data in &test_data.shortest_path {
               println!(
                   "ShortestPath : unique = {} and nshortest = {}",
                   data.unique, data.nshortest
               );
        let fst_res: Result<F> = shortest_path(&test_data.raw, data.nshortest, data.unique);
        match (&data.result, &fst_res) {
            (Ok(fst_expected), Ok(ref fst_shortest)) => {
                let a = isomorphic(fst_expected, fst_shortest)?;
                assert!(
                    a,
                    "{}",
                    error_message_fst!(
                        fst_expected,
                        fst_shortest,
                        format!(
                            "ShortestPath fail for nshortest = {:?} and unique = {:?}",
                            data.nshortest, data.unique
                        )
                    )
                );
            }
            (Ok(_fst_expected), Err(e)) => panic!(
                "ShortestPath fail for nshortest = {:?} and unique = {:?}. Got Err. Expected Ok \n{:?}",
                data.nshortest, data.unique, e
            ),
            (Err(_), Ok(_fst_shortest)) => panic!(
                "ShortestPath fail for nshortest = {:?} and unique = {:?}. Got Ok. Expected Err \n{}",
                data.nshortest, data.unique, _fst_shortest
            ),
            (Err(_), Err(_)) => {
                // Ok
            }
        };
    }

    Ok(())
}
