use std::fmt::Display;

use anyhow::{format_err, Result};
use serde::{Deserialize, Serialize};

use crate::algorithms::{
    determinize::{determinize, DeterminizeType},
    isomorphic,
};
use crate::fst_properties::FstProperties;
use crate::fst_traits::{AllocableFst, MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::semirings::WeaklyDivisibleSemiring;
use crate::semirings::WeightQuantize;
use crate::tests_openfst::FstTestData;
use bitflags::_core::marker::PhantomData;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
pub struct DeterminizeOperationResult {
    det_type: String,
    result: String,
}

pub struct DeterminizeTestData<W, F>
where
    F: SerializableFst<W>,
    W: SerializableSemiring,
{
    det_type: DeterminizeType,
    result: Result<F>,
    w: PhantomData<W>,
}

impl DeterminizeOperationResult {
    pub fn parse<W, F>(&self) -> DeterminizeTestData<W, F>
    where
        F: SerializableFst<W>,
        W: SerializableSemiring,
    {
        DeterminizeTestData {
            det_type: match self.det_type.as_str() {
                "functional" => DeterminizeType::DeterminizeFunctional,
                "nonfunctional" => DeterminizeType::DeterminizeNonFunctional,
                "disambiguate" => DeterminizeType::DeterminizeDisambiguate,
                _ => panic!("Unknown determinize type : {:?}", self.det_type),
            },
            result: match self.result.as_str() {
                "error" => Err(format_err!("lol")),
                _ => F::from_text_string(self.result.as_str()),
            },
            w: PhantomData,
        }
    }
}

pub fn test_determinize<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + AllocableFst<W> + Display,
    W: SerializableSemiring + WeaklyDivisibleSemiring + WeightQuantize,
{
    for determinize_data in &test_data.determinize {
        //        println!("det_type = {:?}", determinize_data.det_type);
        let fst_raw = Arc::new(test_data.raw.clone());
        let fst_res: Result<F> = determinize(fst_raw, determinize_data.det_type.clone());

        match (&determinize_data.result, fst_res) {
            (Ok(fst_expected), Ok(ref fst_determinized)) => {
                if determinize_data.det_type == DeterminizeType::DeterminizeFunctional {
                    assert!(fst_determinized
                        .properties()?
                        .contains(FstProperties::I_DETERMINISTIC));
                }
                let a = isomorphic(fst_expected, fst_determinized)?;
                assert!(
                    a,
                    "{}",
                    error_message_fst!(
                        fst_expected,
                        fst_determinized,
                        format!(
                            "Determinize fail for det_type = {:?} ",
                            determinize_data.det_type
                        )
                    )
                );
            }
            (Ok(_fst_expected), Err(_)) => panic!(
                "Determinize fail for det_type {:?}. Got Err. Expected Ok",
                determinize_data.det_type
            ),
            (Err(_), Ok(_fst_determinized)) => panic!(
                "Determinize fail for det_type {:?}. Got Ok. Expected Err, \n{}",
                determinize_data.det_type, _fst_determinized
            ),
            (Err(_), Err(_)) => {
                // Ok
            }
        };
    }
    Ok(())
}
