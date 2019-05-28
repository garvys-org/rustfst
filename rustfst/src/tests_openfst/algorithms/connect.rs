use failure::Fallible;

use crate::algorithms::connect;
use crate::fst_properties::FstProperties;
use crate::fst_traits::MutableFst;
use crate::fst_traits::TextParser;
use crate::semirings::Semiring;

use crate::tests_openfst::TestData;

pub fn test_connect<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32>,
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
