use std::fmt::Display;
use std::marker::PhantomData;
use std::path::Path;

use anyhow::{format_err, Result};
use serde::{Deserialize, Serialize};

use crate::algorithms::determinize::{determinize_with_config, DeterminizeConfig, DeterminizeType};
use crate::fst_properties::FstProperties;
use crate::fst_traits::{AllocableFst, MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::semirings::WeaklyDivisibleSemiring;
use crate::semirings::WeightQuantize;
use crate::tests_openfst::utils::test_isomorphic_fst;
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct DeterminizeOperationResult {
    det_type: String,
    result_path: String,
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
    pub fn parse<W, F, P>(&self, dir_path: P) -> DeterminizeTestData<W, F>
    where
        F: SerializableFst<W>,
        W: SerializableSemiring,
        P: AsRef<Path>,
    {
        DeterminizeTestData {
            det_type: match self.det_type.as_str() {
                "functional" => DeterminizeType::DeterminizeFunctional,
                "nonfunctional" => DeterminizeType::DeterminizeNonFunctional,
                "disambiguate" => DeterminizeType::DeterminizeDisambiguate,
                _ => panic!("Unknown determinize type : {:?}", self.det_type),
            },
            result: match self.result_path.as_str() {
                "error" => Err(format_err!("lol")),
                _ => F::read(dir_path.as_ref().join(&self.result_path)),
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
        let config = DeterminizeConfig::default().with_det_type(determinize_data.det_type);
        let fst_res: Result<F> = determinize_with_config(&test_data.raw, config);

        match (&determinize_data.result, fst_res) {
            (Ok(fst_expected), Ok(ref fst_determinized)) => {
                if determinize_data.det_type == DeterminizeType::DeterminizeFunctional {
                    let mut fst = fst_determinized.clone();
                    fst.compute_and_update_properties_all()?;
                    assert!(fst.properties().contains(FstProperties::I_DETERMINISTIC));
                }

                test_isomorphic_fst(
                    fst_expected,
                    fst_determinized,
                    format!(
                        "Determinize fail for det_type = {:?} ",
                        determinize_data.det_type
                    ),
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
