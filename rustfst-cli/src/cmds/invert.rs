use rustfst::prelude::*;

use log::debug;

use failure::Fallible;

pub fn invert_cli(path_in: &str, path_out: &str) -> Fallible<()> {
    debug!("Reading FST");
    let mut fst = VectorFst::<TropicalWeight>::read(path_in)?;
    debug!("Running invert algorithm");
    invert(&mut fst);
    debug!("Writing FST");
    fst.write(path_out)?;
    debug!("Done");
    Ok(())
}
