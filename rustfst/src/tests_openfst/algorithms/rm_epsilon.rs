use std::fmt::Display;

use failure::Fallible;

use crate::algorithms::{isomorphic, rm_epsilon};
use crate::fst_impls::VectorFst;
use crate::fst_properties::FstProperties;
use crate::fst_traits::TextParser;
use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::semirings::Semiring;
use crate::semirings::StarSemiring;
use crate::semirings::WeaklyDivisibleSemiring;

use crate::tests_openfst::FstTestData;

pub fn test_rmepsilon<F>(test_data: &FstTestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst + ExpandedFst + Display,
    F::W: 'static + Semiring<Type = f32> + WeaklyDivisibleSemiring + StarSemiring,
{
    // Remove epsilon
    let fst_rmepsilon: VectorFst<_> = rm_epsilon(&test_data.raw).unwrap();
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
