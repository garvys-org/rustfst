extern crate rustfst;
#[macro_use]
extern crate serde_derive;

use std::fs::read_to_string;
use std::string::String;

use rustfst::algorithms::{
    connect, invert, isomorphic, project, push_weights, reverse, rm_epsilon, ProjectType,
    ReweightType,
};
use rustfst::fst_impls::VectorFst;
use rustfst::fst_traits::TextParser;
use rustfst::semirings::{Semiring, TropicalWeight};
use rustfst::Result;

#[derive(Serialize, Deserialize, Debug)]
struct OperationResult {
    result: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ParsedTestData {
    rmepsilon: OperationResult,
    name: String,
    invert: OperationResult,
    weight_type: String,
    raw: OperationResult,
    project_output: OperationResult,
    connect: OperationResult,
    weight_pushing_initial: OperationResult,
    weight_pushing_final: OperationResult,
    project_input: OperationResult,
    reverse: OperationResult,
}

struct TestData<W: Semiring<Type = f32>, F: TextParser<W = W>> {
    rmepsilon: F,
    #[allow(unused)]
    name: String,
    invert: F,
    raw: F,
    project_output: F,
    connect: F,
    weight_pushing_initial: F,
    #[allow(unused)]
    weight_pushing_final: F,
    project_input: F,
    reverse: F,
}

impl<W: Semiring<Type = f32>, F: TextParser<W = W>> TestData<W, F> {
    pub fn new(data: &ParsedTestData) -> Self {
        Self {
            rmepsilon: F::from_text_string(data.rmepsilon.result.as_str()).unwrap(),
            name: data.name.clone(),
            invert: F::from_text_string(data.invert.result.as_str()).unwrap(),
            raw: F::from_text_string(data.raw.result.as_str()).unwrap(),
            project_output: F::from_text_string(data.project_output.result.as_str()).unwrap(),
            connect: F::from_text_string(data.connect.result.as_str()).unwrap(),
            weight_pushing_initial: F::from_text_string(
                data.weight_pushing_initial.result.as_str(),
            )
            .unwrap(),
            weight_pushing_final: F::from_text_string(data.weight_pushing_final.result.as_str())
                .unwrap(),
            project_input: F::from_text_string(data.project_input.result.as_str()).unwrap(),
            reverse: F::from_text_string(data.reverse.result.as_str()).unwrap(),
        }
    }
}

macro_rules! error_message_fst {
    ($fst_ref:expr, $fst:expr, $operation_name:expr) => {
        format!(
            "\nTest {} with pynini failing : \nREF = \n{}\nPRED = \n{}\n",
            $operation_name, $fst_ref, $fst
        )
    };
}

macro_rules! run_test_pynini {
    ($test_name:expr) => {
        let mut absolute_path = std::env::current_dir()?;
        absolute_path.push("fst-test-data");
        absolute_path.push("fst_test_data");
        absolute_path.push($test_name);
        absolute_path.push("metadata.json");

        let string = read_to_string(absolute_path).unwrap();
        let parsed_test_data: ParsedTestData = serde_json::from_str(&string).unwrap();
        let test_data: TestData<TropicalWeight, VectorFst<TropicalWeight>> =
            TestData::new(&parsed_test_data);

        // Remove epsilon
        let fst_rmepsilon: VectorFst<TropicalWeight> = rm_epsilon(&test_data.raw).unwrap();
        assert!(
            isomorphic(&fst_rmepsilon, &test_data.rmepsilon)?,
            "{}",
            error_message_fst!(test_data.rmepsilon, fst_rmepsilon, "RmEpsilon")
        );

        // Invert
        let mut fst_invert = test_data.raw.clone();
        invert(&mut fst_invert);
        assert_eq!(
            test_data.invert,
            fst_invert,
            "{}",
            error_message_fst!(test_data.invert, fst_invert, "Invert")
        );

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

        // Project input
        let mut fst_project_input = test_data.raw.clone();
        project(&mut fst_project_input, ProjectType::ProjectInput);
        assert_eq!(
            test_data.project_input,
            fst_project_input,
            "{}",
            error_message_fst!(test_data.project_input, fst_project_input, "Project Input")
        );

        // Reverse
        let fst_reverse: VectorFst<TropicalWeight> = reverse(&test_data.raw).unwrap();
        assert!(
            isomorphic(&test_data.reverse, &fst_reverse)?,
            "{}",
            error_message_fst!(test_data.reverse, fst_reverse, "Reverse")
        );

        // Connect
        let mut fst_connect = test_data.raw.clone();
        connect(&mut fst_connect)?;
        assert_eq!(
            test_data.connect,
            fst_connect,
            "{}",
            error_message_fst!(test_data.connect, fst_connect, "Connect")
        );

        // Weight pushing initial
        let mut fst_weight_push_initial = test_data.raw.clone();
        push_weights(
            &mut fst_weight_push_initial,
            ReweightType::ReweightToInitial,
        )?;
        assert_eq!(
            test_data.weight_pushing_initial,
            fst_weight_push_initial,
            "{}",
            error_message_fst!(
                test_data.weight_pushing_initial,
                fst_weight_push_initial,
                "Weight Pushing initial"
            )
        );

        // Weight pushing final
        let mut fst_weight_push_final = test_data.raw.clone();
        push_weights(&mut fst_weight_push_final, ReweightType::ReweightToFinal)?;
        assert_eq!(
            test_data.weight_pushing_final,
            fst_weight_push_final,
            "{}",
            error_message_fst!(
                test_data.weight_pushing_final,
                fst_weight_push_final,
                "Weight Pushing final"
            )
        );
    };
}

#[test]
fn test_pynini_fst_000() -> Result<()> {
    run_test_pynini!("fst_000");
    Ok(())
}

#[test]
fn test_pynini_fst_001() -> Result<()> {
    run_test_pynini!("fst_001");
    Ok(())
}

#[test]
fn test_pynini_fst_002() -> Result<()> {
    run_test_pynini!("fst_003");
    Ok(())
}

#[test]
fn test_pynini_fst_003() -> Result<()> {
    run_test_pynini!("fst_003");
    Ok(())
}

#[test]
fn test_pynini_fst_004() -> Result<()> {
    run_test_pynini!("fst_004");
    Ok(())
}
