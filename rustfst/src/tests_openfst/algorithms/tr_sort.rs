use std::fmt::Display;

use anyhow::Result;

use crate::algorithms::tr_compares::{ilabel_compare, olabel_compare};
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
    tr_sort(&mut fst_trsort, ilabel_compare);
    assert!(fst_trsort
        .properties()?
        .contains(FstProperties::I_LABEL_SORTED));
    Ok(())
}

pub fn test_trsort_olabel<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring + WeightQuantize,
{
    let mut fst_trsort = test_data.raw.clone();
    tr_sort(&mut fst_trsort, olabel_compare);
    assert!(fst_trsort
        .properties()?
        .contains(FstProperties::O_LABEL_SORTED));
    Ok(())
}
