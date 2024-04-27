use nom::combinator::verify;
use nom::multi::count;
use nom::IResult;
use std::hash::BuildHasher;

use crate::parsers::bin_fst::fst_header::OpenFstString;
use crate::parsers::nom_utils::NomCustomError;
use crate::parsers::{parse_bin_i32, parse_bin_i64};
use crate::parsers::{write_bin_i32, write_bin_i64};
use crate::{Label, SymbolTable};
use anyhow::Result;
use std::io::Write;

static SYMBOL_TABLE_MAGIC_NUMBER: i32 = 2_125_658_996;

fn parse_row_symt(i: &[u8]) -> IResult<&[u8], (i64, OpenFstString), NomCustomError<&[u8]>> {
    let (i, symbol) = OpenFstString::parse(i)?;
    let (i, key) = parse_bin_i64(i)?;
    Ok((i, (key, symbol)))
}

pub(crate) fn parse_symbol_table_bin(
    i: &[u8],
) -> IResult<&[u8], SymbolTable, NomCustomError<&[u8]>> {
    let (i, _magic_number) = verify(parse_bin_i32, |v| *v == SYMBOL_TABLE_MAGIC_NUMBER)(i)?;
    let (i, _name) = OpenFstString::parse(i)?;
    let (i, _available_key) = parse_bin_i64(i)?;
    let (i, num_symbols) = parse_bin_i64(i)?;
    let (i, pairs_idx_symbols) = count(parse_row_symt, num_symbols as usize)(i)?;

    let mut symt = SymbolTable::empty();
    for (key, symbol) in pairs_idx_symbols.into_iter() {
        let inserted_label = symt.add_symbol(symbol);
        if inserted_label != key as Label {
            return Err(nom::Err::Error(NomCustomError::SymbolTableError(
                format!("SymbolTable must contain increasing labels with no hole. Expected : {} and Got : {}", inserted_label, key)
            )));
        }
    }

    Ok((i, symt))
}

pub(crate) fn write_bin_symt<W: Write, H: BuildHasher>(
    file: &mut W,
    symt: &SymbolTable<H>,
) -> Result<()> {
    write_bin_i32(file, SYMBOL_TABLE_MAGIC_NUMBER)?;
    OpenFstString::new("rustfst_symboltable").write(file)?;
    // TODO: Might not be available
    write_bin_i64(file, symt.len() as i64)?;
    write_bin_i64(file, symt.len() as i64)?;
    for (label, symbol) in symt.iter() {
        OpenFstString::new(symbol).write(file)?;
        write_bin_i64(file, label as i64)?;
    }

    Ok(())
}
