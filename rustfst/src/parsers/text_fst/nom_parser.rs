use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::tab;
use nom::combinator::opt;
use nom::multi::separated_list0;
use nom::sequence::preceded;
use nom::IResult;

use crate::parsers::nom_utils::num;
use crate::parsers::text_fst::parsed_text_fst::{FinalState, RowParsed, Transition};
use crate::semirings::SerializableSemiring;

fn optional_weight<W: SerializableSemiring>(i: &str) -> IResult<&str, Option<W>> {
    opt(preceded(tab, W::parse_text))(i)
}

fn transition<W: SerializableSemiring>(i: &str) -> IResult<&str, RowParsed<W>> {
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

fn final_state<W: SerializableSemiring>(i: &str) -> IResult<&str, RowParsed<W>> {
    let (i, state) = num(i)?;
    let (i, weight) = optional_weight(i)?;
    Ok((i, RowParsed::FinalState(FinalState { state, weight })))
}

fn infinity_final_state<W: SerializableSemiring>(i: &str) -> IResult<&str, RowParsed<W>> {
    let (i, state) = num(i)?;
    let (i, _) = tab(i)?;
    let (i, _) = tag("Infinity")(i)?;
    Ok((i, RowParsed::InfinityFinalState(state)))
}

fn row_parsed<W: SerializableSemiring>(i: &str) -> IResult<&str, RowParsed<W>> {
    alt((transition, infinity_final_state, final_state))(i)
}

pub fn vec_rows_parsed<W: SerializableSemiring>(i: &str) -> IResult<&str, Vec<RowParsed<W>>> {
    separated_list0(tag("\n"), row_parsed)(i)
}
