use std::fmt::Display;

use failure::Fallible;

use crate::algorithms::{isomorphic, rm_epsilon};
use crate::fst_properties::FstProperties;
use crate::fst_traits::{ExpandedFst, MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::semirings::WeaklyDivisibleSemiring;
use crate::tests_openfst::FstTestData;

pub fn test_rmepsilon<F>(test_data: &FstTestData<F>) -> Fallible<()>
where
    F: SerializableFst + MutableFst + ExpandedFst + Display,
    F::W: 'static + SerializableSemiring + WeaklyDivisibleSemiring,
{
    // Remove epsilon
    let mut fst_rmepsilon = test_data.raw.clone();
    rm_epsilon(&mut fst_rmepsilon)?;
    assert!(fst_rmepsilon
        .properties()?
        .contains(FstProperties::NO_EPSILONS));
    assert!(
        isomorphic(&fst_rmepsilon, &test_data.rmepsilon)?,
        "{}",
        error_message_fst!(test_data.rmepsilon, fst_rmepsilon, "RmEpsilon")
    );
    Ok(())
}
