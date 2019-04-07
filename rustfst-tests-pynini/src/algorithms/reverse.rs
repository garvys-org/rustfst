use failure::Fallible;

use rustfst::algorithms::{isomorphic, reverse};
use rustfst::fst_impls::VectorFst;
use rustfst::fst_traits::MutableFst;
use rustfst::fst_traits::TextParser;
use rustfst::semirings::Semiring;
use rustfst::semirings::WeaklyDivisibleSemiring;

use crate::TestData;

pub fn test_reverse<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: 'static + Semiring<Type = f32> + WeaklyDivisibleSemiring,
{
    // Reverse
    let fst_reverse: VectorFst<_> = reverse(&test_data.raw).unwrap();
    assert!(
        isomorphic(&test_data.reverse, &fst_reverse)?,
        "{}",
        error_message_fst!(test_data.reverse, fst_reverse, "Reverse")
    );
    Ok(())
}
