use anyhow::Result;

use rustfst::algorithms::determinize::{DeterminizeConfig, DeterminizeType};
use rustfst::prelude::*;

use crate::unary_fst_algorithm::UnaryFstAlgorithm;

pub struct DeterminizeAlgorithm {
    path_in: String,
    path_out: String,
    det_type: DeterminizeType,
}

impl UnaryFstAlgorithm for DeterminizeAlgorithm {
    fn get_path_in(&self) -> &str {
        self.path_in.as_str()
    }

    fn get_path_out(&self) -> &str {
        self.path_out.as_str()
    }

    fn get_algorithm_name(&self) -> String {
        "determinize".to_string()
    }

    fn run_algorithm(&self, fst: VectorFst<TropicalWeight>) -> Result<VectorFst<TropicalWeight>> {
        let det_config = DeterminizeConfig::default().with_det_type(self.det_type);
        let fst = determinize::determinize_with_config(&fst, det_config)?;
        Ok(fst)
    }
}

impl DeterminizeAlgorithm {
    pub fn new(path_in: &str, path_out: &str, det_type: &str) -> Self {
        let det_type = match det_type {
            "functional" => DeterminizeType::DeterminizeFunctional,
            "nonfunctional" => DeterminizeType::DeterminizeNonFunctional,
            "disambiguate" => DeterminizeType::DeterminizeDisambiguate,
            _ => panic!("Unexpected determinize type : {}", det_type),
        };
        Self {
            path_in: path_in.to_string(),
            path_out: path_out.to_string(),
            det_type,
        }
    }
}
