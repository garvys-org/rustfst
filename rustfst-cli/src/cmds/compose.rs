use anyhow::Result;

use rustfst::algorithms::compose::compose;
use rustfst::fst_impls::VectorFst;
use rustfst::semirings::TropicalWeight;

use crate::binary_fst_algorithm::BinaryFstAlgorithm;
use std::sync::Arc;
use rustfst::fst_traits::ExpandedFst;

pub struct ComposeAlgorithm {
    path_in_1: String,
    path_in_2: String,
    path_out: String,
}

impl BinaryFstAlgorithm for ComposeAlgorithm {
    fn get_path_in_1(&self) -> &str {
        &self.path_in_1
    }

    fn get_path_in_2(&self) -> &str {
        &self.path_in_2
    }

    fn get_path_out(&self) -> &str {
        &self.path_out
    }

    fn get_algorithm_name(&self) -> String {
        "compose".to_string()
    }

    fn run_algorithm(
        &self,
        fst_1: VectorFst<TropicalWeight>,
        fst_2: VectorFst<TropicalWeight>,
    ) -> Result<VectorFst<TropicalWeight>> {
        compose(Arc::new(fst_1), Arc::new(fst_2))
    }
}

impl ComposeAlgorithm {
    pub fn new(path_in_1: &str, path_in_2: &str, path_out: &str) -> Self {
        Self {
            path_in_1: path_in_1.to_string(),
            path_in_2: path_in_2.to_string(),
            path_out: path_out.to_string(),
        }
    }
}
