use std::fmt::Display;

use failure::Fallible;

use crate::algorithms::invert;
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::semirings::WeaklyDivisibleSemiring;
use crate::tests_openfst::FstTestData;

pub fn test_invert<F>(test_data: &FstTestData<F>) -> Fallible<()>
where
    F: SerializableFst + MutableFst + Display,
    F::W: SerializableSemiring + WeaklyDivisibleSemiring,
{
    // Invert
    let mut fst_invert = test_data.raw.clone();
    invert(&mut fst_invert);
    assert_eq!(
        test_data.invert,
        fst_invert,
        "{}",
        error_message_fst!(test_data.invert, fst_invert, "Invert")
    );
    Ok(())
}
