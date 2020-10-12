use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::algorithms::{push, PushType, ReweightType};
use crate::fst_impls::VectorFst;
use crate::fst_traits::SerializableFst;
use crate::semirings::{SerializableSemiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::utils::test_eq_fst;
use crate::tests_openfst::FstTestData;
use std::marker::PhantomData;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct PushOperationResult {
    push_labels: bool,
    push_weights: bool,
    remove_common_affix: bool,
    remove_total_weight: bool,
    reweight_to_final: bool,
    result_path: String,
}

pub struct PushTestData<W, F>
where
    F: SerializableFst<W>,
    W: SerializableSemiring,
{
    pub push_labels: bool,
    pub push_weights: bool,
    pub remove_common_affix: bool,
    pub remove_total_weight: bool,
    pub reweight_to_final: bool,
    pub result: F,
    w: PhantomData<W>,
}

impl PushOperationResult {
    pub fn parse<W, F, P>(&self, dir_path: P) -> PushTestData<W, F>
    where
        F: SerializableFst<W>,
        W: SerializableSemiring,
        P: AsRef<Path>,
    {
        PushTestData {
            push_labels: self.push_labels,
            push_weights: self.push_weights,
            remove_common_affix: self.remove_common_affix,
            remove_total_weight: self.remove_total_weight,
            reweight_to_final: self.reweight_to_final,
            result: F::read(dir_path.as_ref().join(&self.result_path)).unwrap(),
            w: PhantomData,
        }
    }
}

pub fn test_push<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring,
{
    for push_test_data in &test_data.push {
        //        println!(
        //            "Push : {:?}",
        //            vec![
        //                push_test_data.push_weights,
        //                push_test_data.push_labels,
        //                push_test_data.remove_total_weight,
        //                push_test_data.remove_common_affix,
        //                push_test_data.reweight_to_final
        //            ]
        //        );
        let mut push_type = PushType::empty();
        if push_test_data.push_weights {
            push_type.insert(PushType::PUSH_WEIGHTS);
        }
        if push_test_data.push_labels {
            push_type.insert(PushType::PUSH_LABELS);
        }
        if push_test_data.remove_total_weight {
            push_type.insert(PushType::REMOVE_TOTAL_WEIGHT);
        }
        if push_test_data.remove_common_affix {
            push_type.insert(PushType::REMOVE_COMMON_AFFIX);
        }

        let reweight_type = if push_test_data.reweight_to_final {
            ReweightType::ReweightToFinal
        } else {
            ReweightType::ReweightToInitial
        };

        let pushed_fst: VectorFst<W> = push(&test_data.raw, reweight_type, push_type)?;

        test_eq_fst(
            &push_test_data.result,
            &pushed_fst,
            format!(
                "Push failed with parameters {:?}",
                vec![
                    push_test_data.push_weights,
                    push_test_data.push_labels,
                    push_test_data.remove_total_weight,
                    push_test_data.remove_common_affix,
                    push_test_data.reweight_to_final
                ]
            ),
        );
    }
    Ok(())
}
