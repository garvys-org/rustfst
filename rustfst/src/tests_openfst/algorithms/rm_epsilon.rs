use std::fmt::Display;

use anyhow::Result;

use crate::algorithms::{rm_epsilon, RmEpsilonFst};
use crate::fst_impls::VectorFst;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{ExpandedFst, MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::semirings::WeaklyDivisibleSemiring;
use crate::tests_openfst::algorithms::dynamic_fst::compare_fst_static_dynamic;
use crate::tests_openfst::FstTestData;

pub fn test_rmepsilon<F>(test_data: &FstTestData<F>) -> Result<()>
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

pub fn test_rmepsilon_dynamic<W>(test_data: &FstTestData<VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + 'static,
    W::ReverseWeight: 'static,
{
    let rmepsilon_dynamic_fst_openfst = &test_data.rmepsilon.result_dynamic;
    let rmepsilon_dynamic_fst = RmEpsilonFst::new(test_data.raw.clone());
    compare_fst_static_dynamic(rmepsilon_dynamic_fst_openfst, &rmepsilon_dynamic_fst)?;

    Ok(())
}
