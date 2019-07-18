use failure::Fallible;

use rustfst::prelude::*;

use crate::unary_fst_algorithm::UnaryFstAlgorithm;

pub struct PushAlgorithm {
    path_in: String,
    path_out: String,
    push_weights: bool,
    remove_total_weight: bool,
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

    fn run_algorithm(
        &self,
        mut fst: VectorFst<TropicalWeight>,
    ) -> Fallible<VectorFst<TropicalWeight>> {
        if self.push_weights {
            push_weights(&mut fst, self.reweight_type, self.remove_total_weight)?;
        } else {
            unimplemented!()
        }
        Ok(fst)
    }
}

impl PushAlgorithm {
    pub fn new(
        path_in: &str,
        path_out: &str,
        reweight_to_final: bool,
        push_weights: bool,
        remove_total_weight: bool,
    ) -> Self {
        Self {
            path_in: path_in.to_string(),
            path_out: path_out.to_string(),
            push_weights,
            remove_total_weight,
            reweight_type: if reweight_to_final {
                ReweightType::ReweightToFinal
            } else {
                ReweightType::ReweightToInitial
            },
        }
    }
}
