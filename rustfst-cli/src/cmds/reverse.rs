use rustfst::prelude::*;

use anyhow::Result;

use crate::unary_fst_algorithm::UnaryFstAlgorithm;

pub struct ReverseAlgorithm {
    path_in: String,
    path_out: String,
}

impl UnaryFstAlgorithm for ReverseAlgorithm {
    fn get_path_in(&self) -> &str {
        self.path_in.as_str()
    }

    fn get_path_out(&self) -> &str {
        self.path_out.as_str()
    }

    fn get_algorithm_name(&self) -> String {
        "reverse".to_string()
    }

    fn run_algorithm(&self, fst: VectorFst<TropicalWeight>) -> Result<VectorFst<TropicalWeight>> {
        reverse(&fst)
    }
}

impl ReverseAlgorithm {
    pub fn new(path_in: &str, path_out: &str) -> Self {
        Self {
            path_in: path_in.to_string(),
            path_out: path_out.to_string(),
        }
    }
}
