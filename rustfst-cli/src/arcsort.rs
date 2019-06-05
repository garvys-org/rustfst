use rustfst::prelude::*;

use log::debug;

use failure::{bail, Fallible};

pub fn arcsort_cli(path_in: &str, sort_type: &str, path_out: &str) -> Fallible<()> {
    debug!("Reading FST");
    let mut fst = VectorFst::<TropicalWeight>::read(path_in)?;
    debug!("Running Arcsort algorithm");
    let cmp = match sort_type {
        "ilabel" => ilabel_compare,
        "olabel" => olabel_compare,
        _ => bail!("Unknow sort_type : {}", sort_type),
    };
    arc_sort(&mut fst, cmp)?;
    debug!("Writing FST");
    fst.write(path_out)?;
    debug!("Done");
    Ok(())
}
