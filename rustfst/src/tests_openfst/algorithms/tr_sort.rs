use std::fmt::Display;

use anyhow::Result;

use crate::algorithms::tr_compares::{ILabelCompare, OLabelCompare};
use crate::algorithms::tr_sort;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::semirings::WeightQuantize;
use crate::tests_openfst::FstTestData;

pub fn test_trsort_ilabel<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring + WeightQuantize,
{
    let mut fst_trsort = test_data.raw.clone();
    tr_sort(&mut fst_trsort, ILabelCompare {});
    assert!(fst_trsort
        .properties()
        .contains(FstProperties::I_LABEL_SORTED));
    assert_eq!(
        test_data.tr_sort_ilabel.properties(),
        fst_trsort.properties()
    );
    Ok(())
}

pub fn test_trsort_olabel<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring + WeightQuantize,
{
    let mut fst_trsort = test_data.raw.clone();
    tr_sort(&mut fst_trsort, OLabelCompare {});
    assert!(fst_trsort
        .properties()
        .contains(FstProperties::O_LABEL_SORTED));
    assert_eq!(
        test_data.tr_sort_olabel.properties(),
        fst_trsort.properties()
    );
    Ok(())
}
