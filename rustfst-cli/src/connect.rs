use rustfst::prelude::*;

use log::info;

use failure::Fallible;

pub fn connect_cli(path_in: &str, path_out: &str) -> Fallible<()> {
    info!("Connect");
    let mut fst = VectorFst::<TropicalWeight>::read(path_in)?;
    connect(&mut fst)?;
    fst.write(path_out)?;
    Ok(())
}