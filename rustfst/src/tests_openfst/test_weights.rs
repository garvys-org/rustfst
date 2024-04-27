use std::fs::read_to_string;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::semirings::{
    GallicWeight, GallicWeightLeft, GallicWeightMin, GallicWeightRestrict, GallicWeightRight,
    LogWeight, ProductWeight, ReverseBack, SerializableSemiring, StringWeightLeft,
    StringWeightRestrict, StringWeightRight, TropicalWeight, WeightQuantize,
};
use crate::{Tr, KDELTA};

use self::super::get_path_folder;

#[derive(Serialize, Deserialize, Debug)]
pub struct ParsedWeightOperationResult {
    name: String,
    tr_type: String,
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
            tr_type: self.tr_type,
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
    tr_type: String,
    weight_1: W,
    weight_2: W,
    one: W,
    zero: W,
    plus: W,
    times: W,
}

fn do_run_test_openfst_weight<W: SerializableSemiring + WeightQuantize>(
    test_data: ParsedWeightTestData<W>,
) -> Result<()> {
    assert_eq!(W::one(), test_data.one);
    assert_eq!(W::zero(), test_data.zero);
    assert_eq!(
        test_data
            .weight_1
            .times(&test_data.weight_2)?
            .quantize(KDELTA)
            .unwrap(),
        test_data.times.quantize(KDELTA).unwrap()
    );
    assert_eq!(
        test_data
            .weight_1
            .plus(&test_data.weight_2)?
            .quantize(KDELTA)
            .unwrap(),
        test_data.plus.quantize(KDELTA).unwrap()
    );
    assert_eq!(W::weight_type(), test_data.weight_type);
    assert_eq!(Tr::<W>::tr_type(), test_data.tr_type);

    assert_eq!(
        test_data.weight_1.reverse()?.reverse_back()?,
        test_data.weight_1
    );

    Ok(())
}

fn run_test_openfst_weight(test_name: &str) -> Result<()> {
    let mut path_metadata = get_path_folder("weights")?;
    path_metadata.push(format!("{}.json", test_name));

    let string = read_to_string(&path_metadata)
        .map_err(|_| format_err!("Can't open {:?}", &path_metadata))?;
    let parsed_operation_result: ParsedWeightOperationResult =
        serde_json::from_str(&string).unwrap();

    // TODO: Infer the Rust weight type from the serialized weight type.
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
        "left_gallic" => {
            let parsed_test_data =
                parsed_operation_result.parse::<GallicWeightLeft<TropicalWeight>>();
            do_run_test_openfst_weight(parsed_test_data)?;
        }
        "right_gallic" => {
            let parsed_test_data =
                parsed_operation_result.parse::<GallicWeightRight<TropicalWeight>>();
            do_run_test_openfst_weight(parsed_test_data)?;
        }
        "restricted_gallic" => {
            let parsed_test_data =
                parsed_operation_result.parse::<GallicWeightRestrict<TropicalWeight>>();
            do_run_test_openfst_weight(parsed_test_data)?;
        }
        "min_gallic" => {
            let parsed_test_data =
                parsed_operation_result.parse::<GallicWeightMin<TropicalWeight>>();
            do_run_test_openfst_weight(parsed_test_data)?;
        }
        "gallic" => {
            let parsed_test_data = parsed_operation_result.parse::<GallicWeight<TropicalWeight>>();
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
fn test_openfst_weight_001() -> Result<()> {
    run_test_openfst_weight("weight_001")
}

#[test]
fn test_openfst_weight_002() -> Result<()> {
    run_test_openfst_weight("weight_002")
}

#[test]
fn test_openfst_weight_003() -> Result<()> {
    run_test_openfst_weight("weight_003")
}

#[test]
fn test_openfst_weight_004() -> Result<()> {
    run_test_openfst_weight("weight_004")
}

#[test]
fn test_openfst_weight_005() -> Result<()> {
    run_test_openfst_weight("weight_005")
}

#[test]
fn test_openfst_weight_006() -> Result<()> {
    run_test_openfst_weight("weight_006")
}

#[test]
fn test_openfst_weight_007() -> Result<()> {
    run_test_openfst_weight("weight_007")
}

#[test]
fn test_openfst_weight_008() -> Result<()> {
    run_test_openfst_weight("weight_008")
}

#[test]
fn test_openfst_weight_009() -> Result<()> {
    run_test_openfst_weight("weight_009")
}

#[test]
fn test_openfst_weight_010() -> Result<()> {
    run_test_openfst_weight("weight_010")
}

#[test]
fn test_openfst_weight_011() -> Result<()> {
    run_test_openfst_weight("weight_011")
}

#[test]
fn test_openfst_weight_012() -> Result<()> {
    run_test_openfst_weight("weight_012")
}
