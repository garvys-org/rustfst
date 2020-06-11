use anyhow::{bail, Result};

use rustfst::prelude::*;

use crate::unary_fst_algorithm::UnaryFstAlgorithm;

pub struct TrsortAlgorithm {
    path_in: String,
    sort_type: String,
    path_out: String,
}

impl UnaryFstAlgorithm for TrsortAlgorithm {
    fn get_path_in(&self) -> &str {
        self.path_in.as_str()
    }

    fn get_path_out(&self) -> &str {
        self.path_out.as_str()
    }

    fn get_algorithm_name(&self) -> String {
        "tr_sort".to_string()
    }

    fn run_algorithm(
        &self,
        mut fst: VectorFst<TropicalWeight>,
    ) -> Result<VectorFst<TropicalWeight>> {
        match self.sort_type.as_str() {
            "ilabel" => tr_sort(&mut fst, ILabelCompare {}),
            "olabel" => tr_sort(&mut fst, OLabelCompare {}),
            _ => bail!("Unknow sort_type : {}", self.sort_type),
        };
        Ok(fst)
    }
}

impl TrsortAlgorithm {
    pub fn new(path_in: &str, sort_type: &str, path_out: &str) -> Self {
        Self {
            path_in: path_in.to_string(),
            sort_type: sort_type.to_string(),
            path_out: path_out.to_string(),
        }
    }
}
