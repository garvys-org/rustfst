use failure::Fallible;

use rustfst::algorithms::top_sort;
use rustfst::fst_properties::FstProperties;
use rustfst::fst_traits::MutableFst;
use rustfst::fst_traits::TextParser;
use rustfst::semirings::Semiring;

use crate::TestData;

pub fn test_topsort<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32>,
{
    let mut fst_topsort = test_data.raw.clone();
    top_sort(&mut fst_topsort)?;
    if test_data.raw.properties()?.contains(FstProperties::ACYCLIC) {
        let top_sorted = fst_topsort
            .properties()?
            .contains(FstProperties::TOP_SORTED);
        assert!(top_sorted);
    } else {
        // If Acyclic, the fst shouldn't have been modified.
        assert_eq!(
            test_data.raw.clone(),
            fst_topsort,
            "{}",
            error_message_fst!(test_data.topsort, fst_topsort, "TopSort")
        );
    }

    Ok(())
}
