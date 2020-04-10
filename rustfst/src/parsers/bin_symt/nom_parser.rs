use nom::combinator::verify;
use nom::multi::count;
use nom::number::complete::{le_i32, le_i64};
use nom::IResult;

use crate::parsers::bin_fst::fst_header::OpenFstString;
use crate::parsers::bin_fst::utils_serialization::{write_bin_i32, write_bin_i64};
use crate::SymbolTable;
use anyhow::Result;
use std::io::Write;

static SYMBOL_TABLE_MAGIC_NUMBER: i32 = 2_125_658_996;

fn parse_row_symt(i: &[u8]) -> IResult<&[u8], (i64, OpenFstString)> {
    let (i, symbol) = OpenFstString::parse(i)?;
    let (i, key) = le_i64(i)?;
    Ok((i, (key, symbol)))
}

pub(crate) fn parse_symbol_table_bin(i: &[u8]) -> IResult<&[u8], SymbolTable> {
    let (i, _magic_number) = verify(le_i32, |v| *v == SYMBOL_TABLE_MAGIC_NUMBER)(i)?;
    let (i, _name) = OpenFstString::parse(i)?;
    let (i, _available_key) = le_i64(i)?;
    let (i, num_symbols) = le_i64(i)?;
    let (i, pairs_idx_symbols) = count(parse_row_symt, num_symbols as usize)(i)?;

    let mut symt = SymbolTable::empty();
    for (key, symbol) in pairs_idx_symbols.into_iter() {
        symt.add_symbol_key(symbol, key as usize);
    }

    Ok((i, symt))
}

pub(crate) fn write_bin_symt<W: Write>(file: &mut W, symt: &SymbolTable) -> Result<()> {
    write_bin_i32(file, SYMBOL_TABLE_MAGIC_NUMBER)?;
    OpenFstString::new("rustfst_symboltable").write(file)?;
    // TODO: Might not be available
    write_bin_i64(file, symt.len() as i64)?;
    write_bin_i64(file, symt.len() as i64)?;
    for (label, symbol) in symt.iter() {
        OpenFstString::new(symbol).write(file)?;
        write_bin_i64(file, *label as i64)?;
    }

    Ok(())
}
