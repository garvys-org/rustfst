use std::fmt::Display;

use anyhow::Result;

use crate::algorithms::top_sort;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::tests_openfst::FstTestData;

pub fn test_topsort<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring,
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
