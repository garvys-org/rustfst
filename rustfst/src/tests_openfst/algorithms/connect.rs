use std::fmt::Display;

use anyhow::Result;

use crate::algorithms::connect;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::tests_openfst::FstTestData;

pub fn test_connect<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: MutableFst<W> + Display + SerializableFst<W>,
    W: SerializableSemiring,
{
    // Connect
    let mut fst_connect = test_data.raw.clone();
    connect(&mut fst_connect)?;

    assert!(fst_connect
        .properties()?
        .contains(FstProperties::ACCESSIBLE | FstProperties::COACCESSIBLE));

    assert_eq!(
        test_data.connect,
        fst_connect,
        "{}",
        error_message_fst!(test_data.connect, fst_connect, "Connect")
    );
    Ok(())
}
