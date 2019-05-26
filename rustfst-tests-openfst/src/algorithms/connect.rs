use failure::Fallible;

use rustfst::algorithms::connect;
use rustfst::fst_traits::MutableFst;
use rustfst::fst_traits::TextParser;
use rustfst::semirings::Semiring;

use crate::TestData;

pub fn test_connect<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32>,
{
    // Connect
    let mut fst_connect = test_data.raw.clone();
    connect(&mut fst_connect)?;
    assert_eq!(
        test_data.connect,
        fst_connect,
        "{}",
        error_message_fst!(test_data.connect, fst_connect, "Connect")
    );
    Ok(())
}
