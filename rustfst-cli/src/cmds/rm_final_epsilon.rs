use anyhow::Result;

use rustfst::prelude::*;

use crate::unary_fst_algorithm::UnaryFstAlgorithm;

pub struct RmFinalEpsilonAlgorithm {
    path_in: String,
    path_out: String,
}

impl UnaryFstAlgorithm for RmFinalEpsilonAlgorithm {
    fn get_path_in(&self) -> &str {
        self.path_in.as_str()
    }

    fn get_path_out(&self) -> &str {
        self.path_out.as_str()
    }

    fn get_algorithm_name(&self) -> String {
        "rm final epsilon".to_string()
    }

    fn run_algorithm(
        &self,
        mut fst: VectorFst<TropicalWeight>,
    ) -> Result<VectorFst<TropicalWeight>> {
        rm_final_epsilon(&mut fst)?;
        Ok(fst)
    }
}

impl RmFinalEpsilonAlgorithm {
    pub fn new(path_in: &str, path_out: &str) -> Self {
        Self {
            path_in: path_in.to_string(),
            path_out: path_out.to_string(),
        }
    }
}
