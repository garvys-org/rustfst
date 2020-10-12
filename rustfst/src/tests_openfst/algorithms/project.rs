use std::fmt::Display;

use anyhow::Result;

use crate::algorithms::{project, ProjectType};
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::WeaklyDivisibleSemiring;
use crate::semirings::{SerializableSemiring, WeightQuantize};
use crate::tests_openfst::utils::test_eq_fst;
use crate::tests_openfst::FstTestData;

pub fn test_project_output<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring + WeaklyDivisibleSemiring + WeightQuantize,
{
    // Project output
    let mut fst_project_output = test_data.raw.clone();
    project(&mut fst_project_output, ProjectType::ProjectOutput);
    test_eq_fst(
        &test_data.project_output,
        &fst_project_output,
        "Project Output",
    );
    Ok(())
}

pub fn test_project_input<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring + WeaklyDivisibleSemiring + WeightQuantize,
{
    // Project input
    let mut fst_project_input = test_data.raw.clone();
    project(&mut fst_project_input, ProjectType::ProjectInput);
    test_eq_fst(
        &test_data.project_input,
        &fst_project_input,
        "Project Input",
    );

    Ok(())
}
