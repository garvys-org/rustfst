use rustfst::prelude::*;

use log::info;

use failure::Fallible;

pub fn minimize_cli(path_in: &str, allow_nondet: bool, path_out: &str) -> Fallible<()> {
    info!("Minimization");
    let mut fst = VectorFst::<TropicalWeight>::read(path_in)?;
    minimize(&mut fst, allow_nondet)?;
    fst.write(path_out)?;
    Ok(())
}
