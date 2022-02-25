use nom::bytes::complete::tag;
use nom::character::complete::space1;
use nom::multi::many0;
use nom::sequence::terminated;
use nom::IResult;

use crate::parsers::nom_utils::{num, word};
use crate::parsers::text_symt::parsed_text_symt::ParsedTextSymt;
use crate::{Label, Symbol};

fn row(i: &str) -> IResult<&str, (Symbol, Label)> {
    let (i, symbol) = word(i)?;
    let (i, _) = space1(i)?;
    let (i, label) = num(i)?;
    Ok((i, (symbol, label)))
}

fn vec_rows(i: &str) -> IResult<&str, Vec<(Symbol, Label)>> {
    many0(terminated(row, tag("\n")))(i)
}

pub(crate) fn parse_text_symt(i: &str) -> IResult<&str, ParsedTextSymt> {
    let (i, pairs) = vec_rows(i)?;
    Ok((i, ParsedTextSymt { pairs }))
}
