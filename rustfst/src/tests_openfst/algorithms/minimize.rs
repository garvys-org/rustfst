use failure::{Fallible, format_err};
use serde_derive::{Deserialize, Serialize};

use crate::algorithms::{DeterminizeType, isomorphic, minimize};
use crate::fst_properties::FstProperties;
use crate::fst_traits::MutableFst;
use crate::fst_traits::TextParser;
use crate::semirings::Semiring;
use crate::semirings::WeaklyDivisibleSemiring;
use crate::semirings::WeightQuantize;

use crate::tests_openfst::TestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct MinimizeOperationResult {
    allow_nondet: bool,
    result: String,
}

pub struct MinimizeTestData<F>
    where
        F: TextParser,
        F::W: Semiring<Type = f32>,
{
    allow_nondet: bool,
    result: Fallible<F>,
}

impl MinimizeOperationResult {
    pub fn parse<F>(&self) -> MinimizeTestData<F>
        where
            F: TextParser,
            F::W: Semiring<Type = f32>,
    {
        MinimizeTestData {
            allow_nondet: self.allow_nondet,
            result: match self.result.as_str() {
                "error" => Err(format_err!("lol")),
                _ => F::from_text_string(self.result.as_str()),
            },
        }
    }
}

pub fn test_minimize<F>(test_data: &TestData<F>) -> Fallible<()>
    where
        F: TextParser + MutableFst,
        F::W: Semiring<Type = f32> + WeaklyDivisibleSemiring + WeightQuantize + 'static,
{
    for minimize_data in &test_data.minimize {
        println!(
            "Minimize : allow_nondet = {}",
            minimize_data.allow_nondet
        );
        let mut fst_raw = test_data.raw.clone();
        let fst_res: Fallible<F> = minimize(&mut fst_raw, minimize_data.allow_nondet).map(|_| fst_raw);

        match (&minimize_data.result, fst_res) {
            (Ok(fst_expected), Ok(ref fst_minimized)) => {
                assert_eq!(
                    fst_expected, fst_minimized,
                    "{}",
                    error_message_fst!(
                        fst_expected,
                        fst_minimized,
                        format!(
                            "Minimize fail for allow_nondet = {:?} ",
                            minimize_data.allow_nondet
                        )
                    )
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
