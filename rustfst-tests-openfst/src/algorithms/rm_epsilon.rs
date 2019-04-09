use failure::Fallible;

use rustfst::algorithms::{isomorphic, rm_epsilon};
use rustfst::fst_impls::VectorFst;
use rustfst::fst_properties::FstProperties;
use rustfst::fst_traits::TextParser;
use rustfst::fst_traits::{Fst, MutableFst};
use rustfst::semirings::Semiring;
use rustfst::semirings::StarSemiring;
use rustfst::semirings::WeaklyDivisibleSemiring;

use crate::TestData;

pub fn test_rmepsilon<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
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
