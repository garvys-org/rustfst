use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::tab;
use nom::combinator::opt;
use nom::multi::separated_list;
use nom::number::complete::float;
use nom::sequence::preceded;
use nom::IResult;

use crate::parsers::nom_utils::num;
use crate::parsers::text_fst::parsed_text_fst::{FinalState, RowParsed, Transition};

fn optional_weight(i: &str) -> IResult<&str, Option<f32>> {
    opt(preceded(tab, float))(i)
}

fn transition(i: &str) -> IResult<&str, RowParsed> {
    let (i, state) = num(i)?;
    let (i, _) = tab(i)?;
    let (i, nextstate) = num(i)?;
    let (i, _) = tab(i)?;
    let (i, ilabel) = num(i)?;
    let (i, _) = tab(i)?;
    let (i, olabel) = num(i)?;
    let (i, weight) = optional_weight(i)?;

    Ok((
        i,
        RowParsed::Transition(Transition {
            state,
            ilabel,
            olabel,
            weight,
            nextstate,
        }),
    ))
}

fn final_state(i: &str) -> IResult<&str, RowParsed> {
    let (i, state) = num(i)?;
    let (i, weight) = optional_weight(i)?;
    Ok((i, RowParsed::FinalState(FinalState { state, weight })))
}

fn infinity_final_state(i: &str) -> IResult<&str, RowParsed> {
    let (i, state) = num(i)?;
    let (i, _) = tab(i)?;
    let (i, _) = tag("Infinity")(i)?;
    Ok((i, RowParsed::InfinityFinalState(state)))
}

fn row_parsed(i: &str) -> IResult<&str, RowParsed> {
    alt((transition, infinity_final_state, final_state))(i)
}

pub fn vec_rows_parsed(i: &str) -> IResult<&str, Vec<RowParsed>> {
    separated_list(tag("\n"), row_parsed)(i)
}
