use failure::Fallible;
use serde_derive::{Deserialize, Serialize};

use crate::algorithms::{replace};
use crate::fst_impls::VectorFst;
use crate::fst_traits::TextParser;
use crate::semirings::{Semiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct ReplaceOperationResult {
    root: usize,
    label: usize,
    fst_to_replace: String,
    epsilon_on_replace: bool,
    result: String,
}

pub struct ReplaceTestData<F>
    where
        F: TextParser,
        F::W: Semiring<Type = f32>,
{
    pub root: usize,
    pub label: usize,
    pub fst_to_replace: F,
    pub epsilon_on_replace: bool,
    pub result: F,
}

impl ReplaceOperationResult {
    pub fn parse<F>(&self) -> ReplaceTestData<F>
        where
            F: TextParser,
            F::W: Semiring<Type = f32>,
    {
        ReplaceTestData {
            root: self.root,
            label: self.label,
            fst_to_replace: F::from_text_string(self.fst_to_replace.as_str()).unwrap(),
            epsilon_on_replace: self.epsilon_on_replace,
            result: F::from_text_string(self.result.as_str()).unwrap(),
        }
    }
}

pub fn test_replace<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
    where
        W: Semiring<Type = f32> + WeightQuantize + WeaklyDivisibleSemiring + 'static,
        W::ReverseWeight: 'static,
{
    for replace_test_data in &test_data.replace {
        let mut fst_list = vec![];
        fst_list.push((replace_test_data.root, test_data.raw.clone()));
        fst_list.push((replace_test_data.label, replace_test_data.fst_to_replace.clone()));
        let replaced_fst = replace(fst_list, replace_test_data.root, replace_test_data.epsilon_on_replace)?;

        assert_eq!(
            replace_test_data.result,
            replaced_fst,
            "{}",
            error_message_fst!(
                replace_test_data.result,
                replaced_fst,
                format!(
                    "Replace failed with parameters root={:?} label={:?} epsilon_on_replace={:?}",
                    replace_test_data.root,
                    replace_test_data.label,
                    replace_test_data.epsilon_on_replace
                )
            )
        );
    }
    Ok(())
}
