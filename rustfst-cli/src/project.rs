use rustfst::prelude::*;

use log::{debug, info};

use failure::Fallible;

pub fn project_cli(path_in: &str, project_output: bool, path_out: &str) -> Fallible<()> {
    info!("Project");
    debug!("Reading FST");
    let mut fst = VectorFst::<TropicalWeight>::read(path_in)?;
    debug!("Projecting FST");
    let project_type = if project_output {
        ProjectType::ProjectOutput
    } else {
        ProjectType::ProjectInput
    };
    project(&mut fst, project_type);
    debug!("Writing FST");
    fst.write(path_out)?;
    Ok(())
}
