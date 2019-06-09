use failure::{bail, Fallible};

use rustfst::prelude::*;

use crate::unary_fst_algorithm::UnaryFstAlgorithm;

pub struct ArcsortAlgorithm {
    path_in: String,
    sort_type: String,
    path_out: String,
}

impl UnaryFstAlgorithm for ArcsortAlgorithm {
    fn get_path_in(&self) -> &str {
        self.path_in.as_str()
    }

    fn get_path_out(&self) -> &str {
        self.path_out.as_str()
    }

    fn get_algorithm_name() -> String {
        "arcsort".to_string()
    }

    fn run_algorithm(
        &self,
        mut fst: VectorFst<TropicalWeight>,
    ) -> Fallible<VectorFst<TropicalWeight>> {
        let cmp = match self.sort_type.as_str() {
            "ilabel" => ilabel_compare,
            "olabel" => olabel_compare,
            _ => bail!("Unknow sort_type : {}", self.sort_type),
        };
        arc_sort(&mut fst, cmp);
        Ok(fst)
    }
}

impl ArcsortAlgorithm {
    pub fn new(path_in: &str, sort_type: &str, path_out: &str) -> Self {
        Self {
            path_in: path_in.to_string(),
            sort_type: sort_type.to_string(),
            path_out: path_out.to_string(),
        }
    }
}
