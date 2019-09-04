use failure::Fallible;
use serde_derive::{Deserialize, Serialize};

use crate::algorithms::{push, PushType, ReweightType};
use crate::fst_impls::VectorFst;
use crate::fst_traits::TextParser;
use crate::semirings::{Semiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct PushOperationResult {
    push_labels: bool,
    push_weights: bool,
    remove_common_affix: bool,
    remove_total_weight: bool,
    reweight_to_final: bool,
    result: String,
}

pub struct PushTestData<F>
where
    F: TextParser,
    F::W: Semiring<Type = f32>,
{
    pub push_labels: bool,
    pub push_weights: bool,
    pub remove_common_affix: bool,
    pub remove_total_weight: bool,
    pub reweight_to_final: bool,
    pub result: F,
}

impl PushOperationResult {
    pub fn parse<F>(&self) -> PushTestData<F>
    where
        F: TextParser,
        F::W: Semiring<Type = f32>,
    {
        PushTestData {
            push_labels: self.push_labels,
            push_weights: self.push_weights,
            remove_common_affix: self.remove_common_affix,
            remove_total_weight: self.remove_total_weight,
            reweight_to_final: self.reweight_to_final,
            result: F::from_text_string(self.result.as_str()).unwrap(),
        }
    }
}

pub fn test_push<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: Semiring<Type = f32> + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
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
        assert_eq!(
            push_test_data.result,
            pushed_fst,
            "{}",
            error_message_fst!(
                push_test_data.result,
                pushed_fst,
                format!(
                    "Push failed with parameters {:?}",
                    vec![
                        push_test_data.push_weights,
                        push_test_data.push_labels,
                        push_test_data.remove_total_weight,
                        push_test_data.remove_common_affix,
                        push_test_data.reweight_to_final
                    ]
                )
            )
        );
    }
    Ok(())
}
