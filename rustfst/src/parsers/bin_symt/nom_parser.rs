use nom::combinator::verify;
use nom::multi::count;
use nom::number::complete::{le_i32, le_i64};
use nom::IResult;

use crate::parsers::bin_fst::fst_header::OpenFstString;
use crate::SymbolTable;

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
