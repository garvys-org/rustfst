use rustfst::prelude::*;

use crate::unary_fst_algorithm::UnaryFstAlgorithm;
use anyhow::Result;

pub struct ShortestPathAlgorithm {
    path_in: String,
    unique: bool,
    nshortest: usize,
    path_out: String,
}

impl UnaryFstAlgorithm for ShortestPathAlgorithm {
    fn get_path_in(&self) -> &str {
        self.path_in.as_str()
    }

    fn get_path_out(&self) -> &str {
        self.path_out.as_str()
    }

    fn get_algorithm_name(&self) -> String {
        "shortest path".to_string()
    }

    fn run_algorithm(&self, fst: VectorFst<TropicalWeight>) -> Result<VectorFst<TropicalWeight>> {
        let config = ShortestPathConfig::default()
            .with_nshortest(self.nshortest)
            .with_unique(self.unique);
        shortest_path_with_config(&fst, config)
    }
}

impl ShortestPathAlgorithm {
    pub fn new(path_in: &str, unique: bool, nshortest: usize, path_out: &str) -> Self {
        Self {
            path_in: path_in.to_string(),
            unique,
            nshortest,
            path_out: path_out.to_string(),
        }
    }
}
