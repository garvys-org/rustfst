use std::fmt::Display;

use anyhow::Result;

use crate::algorithms::rm_epsilon::{rm_epsilon, RmEpsilonFst};
use crate::fst_impls::VectorFst;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{ExpandedFst, MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::semirings::WeaklyDivisibleSemiring;
use crate::tests_openfst::algorithms::lazy_fst::compare_fst_static_lazy;
use crate::tests_openfst::FstTestData;

pub fn test_rmepsilon<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring + WeaklyDivisibleSemiring,
{
    // Remove epsilon
    let mut fst_rmepsilon = test_data.raw.clone();
    rm_epsilon(&mut fst_rmepsilon)?;
    assert!(fst_rmepsilon
        .properties()?
        .contains(FstProperties::NO_EPSILONS));
    assert_eq!(
        fst_rmepsilon,
        test_data.rmepsilon.result_static,
        "{}",
        error_message_fst!(
            test_data.rmepsilon.result_static,
            fst_rmepsilon,
            "RmEpsilon"
        )
    );
    Ok(())
}

pub fn test_rmepsilon_lazy<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring,
{
    unimplemented!()
    // let rmepsilon_lazy_fst_openfst = &test_data.rmepsilon.result_lazy;
    // let rmepsilon_lazy_fst = RmEpsilonFst::new(test_data.raw.clone());
    // compare_fst_static_lazy(rmepsilon_lazy_fst_openfst, &rmepsilon_lazy_fst)?;
    //
    // Ok(())
}
