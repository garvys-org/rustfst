use rustfst::prelude::*;

use log::debug;

use failure::Fallible;

pub fn connect_cli(path_in: &str, path_out: &str) -> Fallible<()> {
    debug!("Reading FST");
    let mut fst = VectorFst::<TropicalWeight>::read(path_in)?;
    debug!("Running Connect algorithm");
    connect(&mut fst)?;
    debug!("Writing FST");
    fst.write(path_out)?;
    debug!("Done");
    Ok(())
}
