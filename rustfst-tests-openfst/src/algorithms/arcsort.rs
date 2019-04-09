use failure::Fallible;

use rustfst::algorithms::arc_compares::{ilabel_compare, olabel_compare};
use rustfst::algorithms::arc_sort;
use rustfst::fst_properties::FstProperties;
use rustfst::fst_traits::MutableFst;
use rustfst::fst_traits::TextParser;
use rustfst::semirings::Semiring;
use rustfst::semirings::WeightQuantize;

use crate::TestData;

pub fn test_arcsort_ilabel<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeightQuantize,
{
    let mut fst_arcsort = test_data.raw.clone();
    arc_sort(&mut fst_arcsort, ilabel_compare)?;
    assert!(fst_arcsort
        .properties()?
        .contains(FstProperties::I_LABEL_SORTED));
    assert_eq!(
        test_data.arcsort_ilabel,
        fst_arcsort,
        "{}",
        error_message_fst!(
            test_data.arc_map_output_epsilon,
            fst_arcsort,
            "ArcSort ilabel"
        )
    );
    Ok(())
}

pub fn test_arcsort_olabel<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeightQuantize,
{
    let mut fst_arcsort = test_data.raw.clone();
    arc_sort(&mut fst_arcsort, olabel_compare)?;
    assert!(fst_arcsort
        .properties()?
        .contains(FstProperties::O_LABEL_SORTED));
    assert_eq!(
        test_data.arcsort_olabel,
        fst_arcsort,
        "{}",
        error_message_fst!(
            test_data.arc_map_output_epsilon,
            fst_arcsort,
            "ArcSort olabel"
        )
    );
    Ok(())
}
