use std::fmt::Display;

use anyhow::Result;

use crate::algorithms::tr_compares::{ilabel_compare, olabel_compare};
use crate::algorithms::tr_sort;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::semirings::WeightQuantize;
use crate::tests_openfst::FstTestData;

pub fn test_trsort_ilabel<F>(test_data: &FstTestData<F>) -> Result<()>
where
    F: SerializableFst + MutableFst + Display,
    F::W: SerializableSemiring + WeightQuantize,
{
    let mut fst_trsort = test_data.raw.clone();
    tr_sort(&mut fst_trsort, ilabel_compare);
    assert!(fst_trsort
        .properties()?
        .contains(FstProperties::I_LABEL_SORTED));
    assert_eq!(
        test_data.tr_sort_ilabel,
        fst_trsort,
        "{}",
        error_message_fst!(test_data.tr_map_output_epsilon, fst_trsort, "TrSort ilabel")
    );
    Ok(())
}

pub fn test_trsort_olabel<F>(test_data: &FstTestData<F>) -> Result<()>
where
    F: SerializableFst + MutableFst + Display,
    F::W: SerializableSemiring + WeightQuantize,
{
    let mut fst_trsort = test_data.raw.clone();
    tr_sort(&mut fst_trsort, olabel_compare);
    assert!(fst_trsort
        .properties()?
        .contains(FstProperties::O_LABEL_SORTED));
    assert_eq!(
        test_data.tr_sort_olabel,
        fst_trsort,
        "{}",
        error_message_fst!(test_data.tr_map_output_epsilon, fst_trsort, "TrSort olabel")
    );
    Ok(())
}
