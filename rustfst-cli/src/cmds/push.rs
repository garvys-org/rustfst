use anyhow::Result;

use rustfst::prelude::*;

use crate::unary_fst_algorithm::UnaryFstAlgorithm;

pub struct PushAlgorithm {
    path_in: String,
    path_out: String,
    push_type: PushType,
    reweight_type: ReweightType,
}

impl UnaryFstAlgorithm for PushAlgorithm {
    fn get_path_in(&self) -> &str {
        self.path_in.as_str()
    }

    fn get_path_out(&self) -> &str {
        self.path_out.as_str()
    }

    fn get_algorithm_name(&self) -> String {
        "push".to_string()
    }

    fn run_algorithm(&self, fst: VectorFst<TropicalWeight>) -> Result<VectorFst<TropicalWeight>> {
        push(&fst, self.reweight_type, self.push_type)
    }
}

impl PushAlgorithm {
    pub fn new(
        path_in: &str,
        path_out: &str,
        reweight_to_final: bool,
        push_weights: bool,
        push_labels: bool,
        remove_total_weight: bool,
        remove_common_affix: bool,
    ) -> Self {
        let mut push_type = PushType::empty();
        if push_weights {
            push_type.insert(PushType::PUSH_WEIGHTS);
        }
        if push_labels {
            push_type.insert(PushType::PUSH_LABELS);
        }
        if remove_total_weight {
            push_type.insert(PushType::REMOVE_TOTAL_WEIGHT);
        }
        if remove_common_affix {
            push_type.insert(PushType::REMOVE_COMMON_AFFIX);
        }
        Self {
            path_in: path_in.to_string(),
            path_out: path_out.to_string(),
            push_type,
            reweight_type: if reweight_to_final {
                ReweightType::ReweightToFinal
            } else {
                ReweightType::ReweightToInitial
            },
        }
    }
}
