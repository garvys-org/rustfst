use rustfst::prelude::*;

use log::debug;

use failure::Fallible;

pub fn topsort_cli(path_in: &str, path_out: &str) -> Fallible<()> {
    debug!("Reading FST");
    let mut fst = VectorFst::<TropicalWeight>::read(path_in)?;
    debug!("Running topsort algorithm");
    top_sort(&mut fst)?;
    debug!("Writing FST");
    fst.write(path_out)?;
    debug!("Done");
    Ok(())
}
