use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

use crate::algorithms::{replace, ReplaceFst};
use crate::fst_impls::VectorFst;
use crate::fst_traits::SerializableFst;
use crate::semirings::{SerializableSemiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::FstTestData;

use super::lazy_fst::compare_fst_static_lazy;

#[derive(Serialize, Deserialize, Debug)]
pub struct ReplaceOperationResult {
    root: usize,
    label_fst_pairs: Vec<(usize, String)>,
    epsilon_on_replace: bool,
    result: String,
}

pub struct ReplaceTestData<F>
where
    F: SerializableFst,
    F::W: SerializableSemiring,
{
    pub root: usize,
    pub label_fst_pairs: Vec<(usize, F)>,
    pub epsilon_on_replace: bool,
    pub result: F,
}

impl ReplaceOperationResult {
    pub fn parse<F>(&self) -> ReplaceTestData<F>
    where
        F: SerializableFst,
        F::W: SerializableSemiring,
    {
        ReplaceTestData {
            root: self.root,
            label_fst_pairs: self
                .label_fst_pairs
                .iter()
                .map(|v| (v.0, F::from_text_string(v.1.as_str()).unwrap()))
                .collect(),
            epsilon_on_replace: self.epsilon_on_replace,
            result: F::from_text_string(self.result.as_str()).unwrap(),
        }
    }
}

pub fn test_replace<W>(test_data: &FstTestData<VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
{
    unimplemented!()
    // for replace_test_data in &test_data.replace {
    //     let mut fst_list = vec![];
    //     fst_list.push((replace_test_data.root, test_data.raw.clone()));
    //     fst_list.extend_from_slice(replace_test_data.label_fst_pairs.as_slice());
    //     let replaced_fst: VectorFst<_> = replace(
    //         fst_list.clone(),
    //         replace_test_data.root,
    //         replace_test_data.epsilon_on_replace,
    //     )?;
    //
    //     // Try givinf borrowed fst as parameters.
    //     let fst_list_2: Vec<(usize, &VectorFst<W>)> =
    //         fst_list.iter().map(|v| (v.0, &v.1)).collect();
    //     let _replaced_fst_2: VectorFst<W> = replace::<VectorFst<_>, _, _>(
    //         fst_list_2,
    //         replace_test_data.root,
    //         replace_test_data.epsilon_on_replace,
    //     )?;
    //
    //     assert_eq!(
    //         replace_test_data.result,
    //         replaced_fst,
    //         "{}",
    //         error_message_fst!(
    //             replace_test_data.result,
    //             replaced_fst,
    //             format!(
    //                 "Replace failed with parameters root={:?} epsilon_on_replace={:?}",
    //                 replace_test_data.root, replace_test_data.epsilon_on_replace
    //             )
    //         )
    //     );
    // }
    // Ok(())
}

pub fn test_replace_lazy<W>(test_data: &FstTestData<VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
{
    unimplemented!()
    // for replace_test_data in &test_data.replace {
    //     let mut fst_list = vec![];
    //     fst_list.push((replace_test_data.root, test_data.raw.clone()));
    //     fst_list.extend_from_slice(replace_test_data.label_fst_pairs.as_slice());
    //
    //     let fst_list_2: Vec<(usize, &VectorFst<W>)> =
    //         fst_list.iter().map(|v| (v.0, &v.1)).collect();
    //     let replaced_static_fst: VectorFst<_> = replace::<VectorFst<_>, _, _>(
    //         fst_list_2,
    //         replace_test_data.root,
    //         replace_test_data.epsilon_on_replace,
    //     )?;
    //
    //     let replaced_lazy_fst = ReplaceFst::new(
    //         fst_list,
    //         replace_test_data.root,
    //         replace_test_data.epsilon_on_replace,
    //     )?;
    //
    //     compare_fst_static_lazy(&replaced_static_fst, &replaced_lazy_fst)?;
    // }
    // Ok(())
}
