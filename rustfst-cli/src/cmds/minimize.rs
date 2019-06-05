use rustfst::prelude::*;

use log::debug;

use failure::Fallible;

pub fn minimize_cli(path_in: &str, allow_nondet: bool, path_out: &str) -> Fallible<()> {
    debug!("Reading FST");
    let mut fst = VectorFst::<TropicalWeight>::read(path_in)?;
    debug!("Running Minimize algorithm");
    minimize(&mut fst, allow_nondet)?;
    debug!("Writing FST");
    fst.write(path_out)?;
    debug!("Done");
    Ok(())
}
