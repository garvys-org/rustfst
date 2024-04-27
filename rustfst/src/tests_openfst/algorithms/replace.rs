use crate::Label;
use crate::StateId;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use crate::algorithms::replace::{replace, ReplaceFst};
use crate::fst_impls::VectorFst;
use crate::fst_traits::SerializableFst;
use crate::semirings::{SerializableSemiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::FstTestData;

use crate::algorithms::fst_convert_from_ref;
use crate::tests_openfst::utils::test_eq_fst;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct ReplaceOperationResult {
    root: Label,
    label_fst_pairs_path: Vec<(Label, String)>,
    epsilon_on_replace: bool,
    result_path: String,
}

pub struct ReplaceTestData<W, F>
where
    F: SerializableFst<W>,
    W: SerializableSemiring,
{
    pub root: Label,
    pub label_fst_pairs: Vec<(Label, F)>,
    pub epsilon_on_replace: bool,
    pub result: F,
    w: PhantomData<W>,
}

impl ReplaceOperationResult {
    pub fn parse<W, F, P>(&self, dir_path: P) -> ReplaceTestData<W, F>
    where
        F: SerializableFst<W>,
        W: SerializableSemiring,
        P: AsRef<Path>,
    {
        ReplaceTestData {
            root: self.root,
            label_fst_pairs: self
                .label_fst_pairs_path
                .iter()
                .map(|v| (v.0, F::read(dir_path.as_ref().join(&v.1)).unwrap()))
                .collect(),
            epsilon_on_replace: self.epsilon_on_replace,
            result: F::read(dir_path.as_ref().join(&self.result_path)).unwrap(),
            w: PhantomData,
        }
    }
}

pub fn test_replace<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
{
    for replace_test_data in &test_data.replace {
        let mut fst_list = vec![];
        fst_list.push((replace_test_data.root as StateId, test_data.raw.clone()));
        fst_list.extend_from_slice(replace_test_data.label_fst_pairs.as_slice());
        let replaced_fst: VectorFst<_> = replace(
            fst_list.clone(),
            replace_test_data.root,
            replace_test_data.epsilon_on_replace,
        )?;

        // Try givinf borrowed fst as parameters.
        let fst_list_2: Vec<(Label, &VectorFst<W>)> =
            fst_list.iter().map(|v| (v.0, &v.1)).collect();
        let _replaced_fst_2: VectorFst<W> = replace::<_, VectorFst<_>, _, _>(
            fst_list_2,
            replace_test_data.root,
            replace_test_data.epsilon_on_replace,
        )?;

        test_eq_fst(
            &replace_test_data.result,
            &replaced_fst,
            format!(
                "Replace failed with parameters root={:?} epsilon_on_replace={:?}",
                replace_test_data.root, replace_test_data.epsilon_on_replace
            ),
        );
    }
    Ok(())
}

pub fn test_replace_lazy<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
{
    for replace_test_data in &test_data.replace {
        let mut fst_list = vec![];
        fst_list.push((replace_test_data.root, test_data.raw.clone()));
        fst_list.extend_from_slice(replace_test_data.label_fst_pairs.as_slice());

        let fst_list_2: Vec<(Label, &VectorFst<W>)> =
            fst_list.iter().map(|v| (v.0, &v.1)).collect();
        let replaced_static_fst: VectorFst<_> = replace::<_, VectorFst<_>, _, _>(
            fst_list_2,
            replace_test_data.root,
            replace_test_data.epsilon_on_replace,
        )?;

        let replaced_lazy_fst: VectorFst<_> = fst_convert_from_ref(&ReplaceFst::new(
            fst_list,
            replace_test_data.root,
            replace_test_data.epsilon_on_replace,
        )?);

        test_eq_fst(&replaced_static_fst, &replaced_lazy_fst, "Replace lazy");
    }
    Ok(())
}
