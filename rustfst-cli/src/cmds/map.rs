use failure::{bail, Fallible};

use rustfst::prelude::*;

use crate::unary_fst_algorithm::UnaryFstAlgorithm;

pub struct MapAlgorithm {
    path_in: String,
    map_type: String,
    path_out: String,
}

impl UnaryFstAlgorithm for MapAlgorithm {
    fn get_path_in(&self) -> &str {
        self.path_in.as_str()
    }

    fn get_path_out(&self) -> &str {
        self.path_out.as_str()
    }

    fn get_algorithm_name(&self) -> String {
        format!("map {}", self.map_type)
    }

    fn run_algorithm(
        &self,
        mut fst: VectorFst<TropicalWeight>,
    ) -> Fallible<VectorFst<TropicalWeight>> {
        match self.map_type.as_str() {
            "arc_unique" => {
                arc_unique(&mut fst);
                Ok(fst)
            }
            "arc_sum" => {
                arc_sum(&mut fst);
                Ok(fst)
            }
            _ => bail!(
                "Internal error. Should never reach that line. Map type not supported = {}",
                self.map_type
            ),
        }
    }
}

impl MapAlgorithm {
    pub fn new(path_in: &str, map_type: &str, path_out: &str) -> Self {
        Self {
            path_in: path_in.to_string(),
            map_type: map_type.to_string(),
            path_out: path_out.to_string(),
        }
    }
}
