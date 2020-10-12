use rustfst::prelude::*;

use crate::unary_fst_algorithm::UnaryFstAlgorithm;
use anyhow::Result;

pub struct MinimizeAlgorithm {
    path_in: String,
    allow_nondet: bool,
    path_out: String,
}

impl UnaryFstAlgorithm for MinimizeAlgorithm {
    fn get_path_in(&self) -> &str {
        self.path_in.as_str()
    }

    fn get_path_out(&self) -> &str {
        self.path_out.as_str()
    }

    fn get_algorithm_name(&self) -> String {
        "minimize".to_string()
    }

    fn run_algorithm(
        &self,
        mut fst: VectorFst<TropicalWeight>,
    ) -> Result<VectorFst<TropicalWeight>> {
        let config = MinimizeConfig::default().with_allow_nondet(self.allow_nondet);
        minimize_with_config(&mut fst, config)?;
        Ok(fst)
    }
}

impl MinimizeAlgorithm {
    pub fn new(path_in: &str, allow_nondet: bool, path_out: &str) -> Self {
        Self {
            path_in: path_in.to_string(),
            allow_nondet,
            path_out: path_out.to_string(),
        }
    }
}
