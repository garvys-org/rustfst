#![cfg(test)]

#[macro_use]
mod macros;

mod algorithms;

use std::collections::HashMap;
use std::fs::read_to_string;
use std::string::String;

use serde_derive::{Deserialize, Serialize};

use failure::{bail, Fail, Fallible};

use rustfst::fst_impls::VectorFst;
use rustfst::fst_properties::FstProperties;
use rustfst::fst_traits::{MutableFst, TextParser};
use rustfst::semirings::{
    LogWeight, Semiring, StarSemiring, TropicalWeight, WeaklyDivisibleSemiring, WeightQuantize,
};

use crate::algorithms::{
    arc_map::{
        test_arc_map_identity, test_arc_map_input_epsilon, test_arc_map_invert,
        test_arc_map_output_epsilon, test_arc_map_plus, test_arc_map_quantize,
        test_arc_map_rmweight, test_arc_map_times,
    },
    arcsort::{test_arcsort_ilabel, test_arcsort_olabel},
    connect::test_connect,
    determinize::{test_determinize, DeterminizeOperationResult, DeterminizeTestData},
    encode::{test_encode, test_encode_decode, EncodeOperationResult, EncodeTestData},
    inverse::test_invert,
    project::{test_project_input, test_project_output},
    properties::{parse_fst_properties, test_fst_properties},
    reverse::test_reverse,
    rm_epsilon::test_rmepsilon,
    state_map::{test_state_map_arc_sum, test_state_map_arc_unique},
    topsort::test_topsort,
    weight_pushing::{test_weight_pushing_final, test_weight_pushing_initial},
};

#[derive(Serialize, Deserialize, Debug)]
struct OperationResult {
    result: String,
}

impl OperationResult {
    fn parse<F>(&self) -> F
    where
        F: TextParser,
        F::W: Semiring<Type = f32>,
    {
        F::from_text_string(self.result.as_str()).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParsedTestData {
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
    arc_map_identity: OperationResult,
    arc_map_rmweight: OperationResult,
    arc_map_invert: OperationResult,
    arc_map_input_epsilon: OperationResult,
    arc_map_output_epsilon: OperationResult,
    arc_map_plus: OperationResult,
    arc_map_times: OperationResult,
    arc_map_quantize: OperationResult,
    encode: Vec<EncodeOperationResult>,
    encode_decode: Vec<EncodeOperationResult>,
    state_map_arc_sum: OperationResult,
    state_map_arc_unique: OperationResult,
    determinize: Vec<DeterminizeOperationResult>,
    arcsort_ilabel: OperationResult,
    arcsort_olabel: OperationResult,
    topsort: OperationResult,
    fst_properties: HashMap<String, bool>,
}

pub struct TestData<F>
where
    F: TextParser,
    F::W: Semiring<Type = f32>,
{
    pub rmepsilon: F,
    #[allow(unused)]
    pub name: String,
    pub invert: F,
    pub raw: F,
    pub project_output: F,
    pub connect: F,
    pub weight_pushing_initial: F,
    pub weight_pushing_final: F,
    pub project_input: F,
    pub reverse: F,
    pub arc_map_identity: F,
    pub arc_map_rmweight: F,
    pub arc_map_invert: F,
    pub arc_map_input_epsilon: F,
    pub arc_map_output_epsilon: F,
    pub arc_map_plus: F,
    pub arc_map_times: F,
    pub arc_map_quantize: F,
    pub encode: Vec<EncodeTestData<F>>,
    pub encode_decode: Vec<EncodeTestData<F>>,
    pub state_map_arc_sum: F,
    pub state_map_arc_unique: F,
    pub determinize: Vec<DeterminizeTestData<F>>,
    pub arcsort_ilabel: F,
    pub arcsort_olabel: F,
    pub topsort: F,
    pub fst_properties: FstProperties,
}

impl<F> TestData<F>
where
    F: TextParser,
    F::W: Semiring<Type = f32>,
{
    pub fn new(data: &ParsedTestData) -> Self {
        Self {
            rmepsilon: data.rmepsilon.parse(),
            name: data.name.clone(),
            invert: data.invert.parse(),
            raw: data.raw.parse(),
            project_output: data.project_output.parse(),
            connect: data.connect.parse(),
            weight_pushing_initial: data.weight_pushing_initial.parse(),
            weight_pushing_final: data.weight_pushing_final.parse(),
            project_input: data.project_input.parse(),
            reverse: data.reverse.parse(),
            arc_map_identity: data.arc_map_identity.parse(),
            arc_map_rmweight: data.arc_map_rmweight.parse(),
            arc_map_invert: data.arc_map_invert.parse(),
            arc_map_input_epsilon: data.arc_map_input_epsilon.parse(),
            arc_map_output_epsilon: data.arc_map_output_epsilon.parse(),
            arc_map_plus: data.arc_map_plus.parse(),
            arc_map_times: data.arc_map_times.parse(),
            arc_map_quantize: data.arc_map_quantize.parse(),
            encode: data.encode.iter().map(|v| v.parse()).collect(),
            encode_decode: data.encode_decode.iter().map(|v| v.parse()).collect(),
            state_map_arc_sum: data.state_map_arc_sum.parse(),
            state_map_arc_unique: data.state_map_arc_unique.parse(),
            determinize: data.determinize.iter().map(|v| v.parse()).collect(),
            arcsort_ilabel: data.arcsort_ilabel.parse(),
            arcsort_olabel: data.arcsort_olabel.parse(),
            topsort: data.topsort.parse(),
            fst_properties: parse_fst_properties(&data.fst_properties),
        }
    }
}

fn run_test_pynini(test_name: &str) -> Fallible<()> {
    let mut absolute_path = std::env::current_dir()?;
    absolute_path.push("..");
    absolute_path.push("rustfst-tests-openfst");
//    absolute_path.push("rustfst_tests_data");
    absolute_path.push(test_name);
    absolute_path.push("metadata.json");

    let string = read_to_string(absolute_path).unwrap();
    let parsed_test_data: ParsedTestData = serde_json::from_str(&string).unwrap();

    match parsed_test_data.weight_type.as_str() {
        "tropical" | "standard" => {
            let test_data: TestData<VectorFst<TropicalWeight>> = TestData::new(&parsed_test_data);
            do_run_test_pynini(&test_data)?;
        }

        "log" => {
            let test_data: TestData<VectorFst<LogWeight>> = TestData::new(&parsed_test_data);
            do_run_test_pynini(&test_data)?;
        }
        _ => bail!("Weight type unknown : {:?}", parsed_test_data.weight_type),
    };

    Ok(())
}

fn do_run_test_pynini<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: 'static + Semiring<Type = f32> + StarSemiring + WeaklyDivisibleSemiring + WeightQuantize,
{
    test_rmepsilon(&test_data)?;

    test_invert(&test_data)?;

    test_project_input(&test_data)?;

    test_project_output(&test_data)?;

    test_reverse(&test_data)?;

    test_connect(&test_data)?;

    test_weight_pushing_initial(&test_data)?;

    test_weight_pushing_final(&test_data)?;

    test_arc_map_identity(&test_data)?;

    test_arc_map_rmweight(&test_data)?;

    test_arc_map_invert(&test_data)?;

    test_arc_map_input_epsilon(&test_data)?;

    test_arc_map_output_epsilon(&test_data)?;

    test_arc_map_plus(&test_data)?;

    test_arc_map_times(&test_data)?;

    test_arc_map_quantize(&test_data)?;

    test_encode(&test_data)?;

    test_encode_decode(&test_data)?;

    test_state_map_arc_sum(&test_data)?;

    test_state_map_arc_unique(&test_data)?;

    test_determinize(&test_data)?;

    test_arcsort_ilabel(&test_data)?;

    test_arcsort_olabel(&test_data)?;

    test_topsort(&test_data)?;

    test_fst_properties(&test_data)?;

    Ok(())
}

pub struct ExitFailure(failure::Error);

/// Prints a list of causes for this Error, along with any backtrace
/// information collected by the Error (if RUST_BACKTRACE=1).
impl std::fmt::Debug for ExitFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let fail = self.0.as_fail();

        writeln!(f, "{}", &fail)?;

        let mut x: &Fail = fail;
        while let Some(cause) = x.cause() {
            writeln!(f, " -> caused by: {}", &cause)?;
            x = cause;
        }
        if let Some(backtrace) = x.backtrace() {
            writeln!(f, "{:?}", backtrace)?;
        }

        Ok(())
    }
}

impl<T: Into<failure::Error>> From<T> for ExitFailure {
    fn from(t: T) -> Self {
        ExitFailure(t.into())
    }
}

#[test]
fn test_pynini_fst_000() -> Result<(), ExitFailure> {
    run_test_pynini("fst_000").map_err(|v| v.into())
}

#[test]
fn test_pynini_fst_001() -> Result<(), ExitFailure> {
    run_test_pynini("fst_001").map_err(|v| v.into())
}

#[test]
fn test_pynini_fst_002() -> Result<(), ExitFailure> {
    run_test_pynini("fst_002").map_err(|v| v.into())
}

#[test]
fn test_pynini_fst_003() -> Result<(), ExitFailure> {
    run_test_pynini("fst_003").map_err(|v| v.into())
}

#[test]
fn test_pynini_fst_004() -> Result<(), ExitFailure> {
    run_test_pynini("fst_004").map_err(|v| v.into())
}

#[test]
fn test_pynini_fst_005() -> Result<(), ExitFailure> {
    run_test_pynini("fst_005").map_err(|v| v.into())
}

#[test]
fn test_pynini_fst_006() -> Result<(), ExitFailure> {
    run_test_pynini("fst_006").map_err(|v| v.into())
}

#[test]
fn test_pynini_fst_007() -> Result<(), ExitFailure> {
    run_test_pynini("fst_007").map_err(|v| v.into())
}
