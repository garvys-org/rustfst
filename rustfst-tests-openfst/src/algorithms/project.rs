use failure::Fallible;

use rustfst::algorithms::{project, ProjectType};
use rustfst::fst_traits::MutableFst;
use rustfst::fst_traits::TextParser;
use rustfst::semirings::Semiring;
use rustfst::semirings::WeaklyDivisibleSemiring;

use crate::TestData;

pub fn test_project_output<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeaklyDivisibleSemiring,
{
    // Project output
    let mut fst_project_output = test_data.raw.clone();
    project(&mut fst_project_output, ProjectType::ProjectOutput);
    assert_eq!(
        test_data.project_output,
        fst_project_output,
        "{}",
        error_message_fst!(
            test_data.project_output,
            fst_project_output,
            "Project Output"
        )
    );
    Ok(())
}

pub fn test_project_input<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeaklyDivisibleSemiring,
{
    // Project input
    let mut fst_project_input = test_data.raw.clone();
    project(&mut fst_project_input, ProjectType::ProjectInput);
    assert_eq!(
        test_data.project_input,
        fst_project_input,
        "{}",
        error_message_fst!(test_data.project_input, fst_project_input, "Project Input")
    );
    Ok(())
}
