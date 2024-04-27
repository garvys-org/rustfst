use std::fs::read_to_string;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::SymbolTable;

use tempfile::tempdir;

use self::super::get_path_folder;

#[derive(Serialize, Deserialize, Debug)]
pub struct ParsedSymtTestData {
    name: String,
    num_symbols: usize,
    symt_bin: String,
    symt_text: String,
}

fn run_test_openfst_symt(test_name: &str) -> Result<()> {
    let absolute_path_folder = get_path_folder(test_name)?;
    let mut path_metadata = absolute_path_folder.clone();
    path_metadata.push("metadata.json");

    let string = read_to_string(&path_metadata)
        .map_err(|_| format_err!("Can't open {:?}", &path_metadata))?;
    let parsed_test_data: ParsedSymtTestData = serde_json::from_str(&string).unwrap();

    let mut path_symt_text = absolute_path_folder.clone();
    path_symt_text.push(parsed_test_data.symt_text);

    let mut path_symt_bin = absolute_path_folder.clone();
    path_symt_bin.push(parsed_test_data.symt_bin);

    let symt = SymbolTable::read_text(path_symt_text)?;

    {
        // Test Parsing Text Symt
        assert_eq!(symt.len(), parsed_test_data.num_symbols);
    }

    {
        // Test serializing and parsing symt
        let dir = tempdir()?;
        let path_symt_serialized = dir.path().join("symt_serialized.txt");
        symt.write_text(&path_symt_serialized)?;
        let symt2 = SymbolTable::read_text(path_symt_serialized)?;
        assert_eq!(symt, symt2);
    }

    let symt_bin = SymbolTable::read(path_symt_bin)?;

    {
        // Test Parsing Bin Symt
        assert_eq!(symt_bin.len(), parsed_test_data.num_symbols);
    }

    {
        // Test serializing and parsing symt bin
        let dir = tempdir()?;
        let path_symt_serialized = dir.path().join("symt_serialized.bin");
        symt.write(&path_symt_serialized)?;
        let symt2 = SymbolTable::read(path_symt_serialized)?;
        assert_eq!(symt_bin, symt2);
    }

    assert_eq!(symt, symt_bin);

    Ok(())
}

#[test]
fn test_openfst_symt_000() -> Result<()> {
    run_test_openfst_symt("symt_000")
}

#[test]
fn test_openfst_symt_001() -> Result<()> {
    run_test_openfst_symt("symt_001")
}

#[test]
fn test_openfst_symt_002() -> Result<()> {
    run_test_openfst_symt("symt_002")
}
