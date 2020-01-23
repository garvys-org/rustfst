use std::fs::read_to_string;

use failure::Fallible;
use serde_derive::{Deserialize, Serialize};

use crate::semirings::{
    LogWeight, ProductWeight, SerializableSemiring, StringWeightLeft, StringWeightRestrict,
    StringWeightRight, TropicalWeight,
};
use crate::Arc;

use self::super::get_path_folder;

#[derive(Serialize, Deserialize, Debug)]
pub struct ParsedWeightOperationResult {
    name: String,
    arc_type: String,
    weight_type: String,
    weight_1: String,
    weight_2: String,
    one: String,
    zero: String,
    plus: String,
    times: String,
}

impl ParsedWeightOperationResult {
    pub fn parse<W: SerializableSemiring>(self) -> ParsedWeightTestData<W> {
        ParsedWeightTestData {
            name: self.name,
            weight_type: self.weight_type,
            arc_type: self.arc_type,
            weight_1: W::parse_text(self.weight_1.as_str()).unwrap().1,
            weight_2: W::parse_text(self.weight_2.as_str()).unwrap().1,
            one: W::parse_text(self.one.as_str()).unwrap().1,
            zero: W::parse_text(self.zero.as_str()).unwrap().1,
            plus: W::parse_text(self.plus.as_str()).unwrap().1,
            times: W::parse_text(self.times.as_str()).unwrap().1,
        }
    }
}

pub struct ParsedWeightTestData<W> {
    #[allow(unused)]
    name: String,
    weight_type: String,
    arc_type: String,
    weight_1: W,
    weight_2: W,
    one: W,
    zero: W,
    plus: W,
    times: W,
}

fn do_run_test_openfst_weight<W: SerializableSemiring>(
    test_data: ParsedWeightTestData<W>,
) -> Fallible<()> {
    assert_eq!(W::one(), test_data.one);
    assert_eq!(W::zero(), test_data.zero);
    assert_eq!(
        test_data.weight_1.times(&test_data.weight_2)?,
        test_data.times
    );
    assert_eq!(
        test_data.weight_1.plus(&test_data.weight_2)?,
        test_data.plus
    );
    assert_eq!(W::weight_type(), test_data.weight_type);
    assert_eq!(Arc::<W>::arc_type(), test_data.arc_type);

    Ok(())
}

fn run_test_openfst_weight(test_name: &str) -> Fallible<()> {
    let absolute_path_folder = get_path_folder("weights")?;
    let mut path_metadata = absolute_path_folder.clone();
    path_metadata.push(format!("{}.json", test_name));

    let string = read_to_string(&path_metadata)
        .map_err(|_| format_err!("Can't open {:?}", &path_metadata))?;
    let parsed_operation_result: ParsedWeightOperationResult =
        serde_json::from_str(&string).unwrap();

    match parsed_operation_result.weight_type.as_str() {
        "tropical" => {
            let parsed_test_data = parsed_operation_result.parse::<TropicalWeight>();
            do_run_test_openfst_weight(parsed_test_data)?;
        }
        "log" => {
            let parsed_test_data = parsed_operation_result.parse::<LogWeight>();
            do_run_test_openfst_weight(parsed_test_data)?;
        }
        "tropical_X_log" => {
            let parsed_test_data =
                parsed_operation_result.parse::<ProductWeight<TropicalWeight, LogWeight>>();
            do_run_test_openfst_weight(parsed_test_data)?;
        }
        "log_X_tropical" => {
            let parsed_test_data =
                parsed_operation_result.parse::<ProductWeight<LogWeight, TropicalWeight>>();
            do_run_test_openfst_weight(parsed_test_data)?;
        }
        "left_string" => {
            let parsed_test_data = parsed_operation_result.parse::<StringWeightLeft>();
            do_run_test_openfst_weight(parsed_test_data)?;
        }
        "right_string" => {
            let parsed_test_data = parsed_operation_result.parse::<StringWeightRight>();
            do_run_test_openfst_weight(parsed_test_data)?;
        }
        "restricted_string" => {
            let parsed_test_data = parsed_operation_result.parse::<StringWeightRestrict>();
            do_run_test_openfst_weight(parsed_test_data)?;
        }
        _ => bail!(
            "Unknown weight_type : {:?}",
            parsed_operation_result.weight_type
        ),
    }

    Ok(())
}

#[test]
fn test_openfst_weight_001() -> Fallible<()> {
    run_test_openfst_weight("weight_001").map_err(|v| v.into())
}

#[test]
fn test_openfst_weight_002() -> Fallible<()> {
    run_test_openfst_weight("weight_002").map_err(|v| v.into())
}

#[test]
fn test_openfst_weight_003() -> Fallible<()> {
    run_test_openfst_weight("weight_003").map_err(|v| v.into())
}

#[test]
fn test_openfst_weight_004() -> Fallible<()> {
    run_test_openfst_weight("weight_004").map_err(|v| v.into())
}

#[test]
fn test_openfst_weight_005() -> Fallible<()> {
    run_test_openfst_weight("weight_005").map_err(|v| v.into())
}

#[test]
fn test_openfst_weight_006() -> Fallible<()> {
    run_test_openfst_weight("weight_006").map_err(|v| v.into())
}

#[test]
fn test_openfst_weight_007() -> Fallible<()> {
    run_test_openfst_weight("weight_007").map_err(|v| v.into())
}
