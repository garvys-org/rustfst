use std::fmt::Display;

use crate::algorithms::{fst_convert, fst_convert_from_ref};
use crate::fst_traits::{AllocableFst, MutableFst, SerializableFst};
use crate::semirings::{SerializableSemiring, WeaklyDivisibleSemiring};
use crate::tests_openfst::FstTestData;

use anyhow::Result;

pub fn test_fst_convert<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display + AllocableFst<W>,
    W: SerializableSemiring + WeaklyDivisibleSemiring,
{
    // Invert
    let fst = test_data.raw.clone();
    let fst_converted_1: F = fst_convert(fst.clone());
    assert_eq!(
        fst_converted_1,
        fst,
        "{}",
        error_message_fst!(fst_converted_1, fst, "fst_convert")
    );

    let fst_converted_2: F = fst_convert_from_ref(&fst);
    assert_eq!(
        fst_converted_2,
        fst,
        "{}",
        error_message_fst!(fst_converted_2, fst, "fst_convert")
    );
    Ok(())
}
