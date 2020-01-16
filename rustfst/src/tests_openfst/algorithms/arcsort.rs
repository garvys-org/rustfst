use std::fmt::Display;

use failure::Fallible;

use crate::algorithms::arc_compares::{ilabel_compare, olabel_compare};
use crate::algorithms::arc_sort;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::semirings::WeightQuantize;
use crate::tests_openfst::FstTestData;

pub fn test_arcsort_ilabel<F>(test_data: &FstTestData<F>) -> Fallible<()>
where
    F: SerializableFst + MutableFst + Display,
    F::W: SerializableSemiring + WeightQuantize,
{
    let mut fst_arcsort = test_data.raw.clone();
    arc_sort(&mut fst_arcsort, ilabel_compare);
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

pub fn test_arcsort_olabel<F>(test_data: &FstTestData<F>) -> Fallible<()>
where
    F: SerializableFst + MutableFst + Display,
    F::W: SerializableSemiring + WeightQuantize,
{
    let mut fst_arcsort = test_data.raw.clone();
    arc_sort(&mut fst_arcsort, olabel_compare);
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
