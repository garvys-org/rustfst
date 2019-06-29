use failure::{bail, Fallible};
use unsafe_unwrap::UnsafeUnwrap;

use rustfst::prelude::*;

use crate::unary_fst_algorithm::UnaryFstAlgorithm;

pub struct MapAlgorithm {
    path_in: String,
    map_type: String,
    weight: Option<f32>,
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
            "arc_sum" => {
                arc_sum(&mut fst);
                Ok(fst)
            }
            "arc_unique" => {
                arc_unique(&mut fst);
                Ok(fst)
            }
            "identity" => {
                let mut mapper = arc_mappers::IdentityArcMapper {};
                arc_map(&mut fst, &mut mapper)?;
                Ok(fst)
            }
            "input_epsilon" => {
                let mut mapper = arc_mappers::InputEpsilonMapper {};
                arc_map(&mut fst, &mut mapper)?;
                Ok(fst)
            }
            "invert" => {
                let mut mapper = arc_mappers::InvertWeightMapper {};
                arc_map(&mut fst, &mut mapper)?;
                Ok(fst)
            }
            "output_epsilon" => {
                let mut mapper = arc_mappers::OutputEpsilonMapper {};
                arc_map(&mut fst, &mut mapper)?;
                Ok(fst)
            }
            "plus" => {
                // Safe because there is a check at parsing time.
                let mut mapper =
                    arc_mappers::PlusMapper::new(unsafe { self.weight.unsafe_unwrap() });
                arc_map(&mut fst, &mut mapper)?;
                Ok(fst)
            }
            "quantize" => {
                // TODO: Handle the delta parameter
                let mut mapper = arc_mappers::QuantizeMapper {};
                arc_map(&mut fst, &mut mapper)?;
                Ok(fst)
            }
            "rmweight" => {
                let mut mapper = arc_mappers::RmWeightMapper {};
                arc_map(&mut fst, &mut mapper)?;
                Ok(fst)
            }
            "times" => {
                // Safe because there is a check at parsing time.
                let mut mapper =
                    arc_mappers::TimesMapper::new(unsafe { self.weight.unsafe_unwrap() });
                arc_map(&mut fst, &mut mapper)?;
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
    pub fn new(path_in: &str, map_type: &str, weight: Option<&str>, path_out: &str) -> Self {
        Self {
            path_in: path_in.to_string(),
            map_type: map_type.to_string(),
            weight: weight.map(|f| f.parse().unwrap()),
            path_out: path_out.to_string(),
        }
    }
}
