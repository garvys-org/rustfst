use anyhow::Result;

use rustfst::prelude::*;

use crate::unary_fst_algorithm::UnaryFstAlgorithm;

#[derive(Debug)]
pub struct ProjectFstAlgorithm {
    path_in: String,
    project_type: ProjectType,
    path_out: String,
}

impl UnaryFstAlgorithm for ProjectFstAlgorithm {
    fn get_path_in(&self) -> &str {
        self.path_in.as_str()
    }

    fn get_path_out(&self) -> &str {
        self.path_out.as_str()
    }

    fn get_algorithm_name(&self) -> String {
        "project".into()
    }

    fn run_algorithm(
        &self,
        mut fst: VectorFst<TropicalWeight>,
    ) -> Result<VectorFst<TropicalWeight>> {
        dbg!(&self);
        project(&mut fst, self.project_type);
        Ok(fst)
    }
}

impl ProjectFstAlgorithm {
    pub fn new(path_in: &str, project_output: bool, path_out: &str) -> ProjectFstAlgorithm {
        dbg!(project_output);
        Self {
            path_in: path_in.to_string(),
            project_type: if project_output {
                ProjectType::ProjectOutput
            } else {
                ProjectType::ProjectInput
            },
            path_out: path_out.to_string(),
        }
    }
}
