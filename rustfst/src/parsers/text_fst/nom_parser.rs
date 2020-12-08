use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::tab;
use nom::combinator::opt;
use nom::multi::separated_list;
use nom::sequence::preceded;
use nom::IResult;

use crate::parsers::nom_utils::num;
use crate::parsers::text_fst::parsed_text_fst::{FinalState, RowParsed, Transition};
use crate::semirings::SerializableSemiring;
use crate::{Label, StateId};

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
            state: state as StateId,
            ilabel: ilabel as Label,
            olabel: olabel as Label,
            weight,
            nextstate: nextstate as StateId,
        }),
    ))
}

fn final_state<W: SerializableSemiring>(i: &str) -> IResult<&str, RowParsed<W>> {
    let (i, state) = num(i)?;
    let (i, weight) = optional_weight(i)?;
    Ok((
        i,
        RowParsed::FinalState(FinalState {
            state: state as StateId,
            weight,
        }),
    ))
}

fn infinity_final_state<W: SerializableSemiring>(i: &str) -> IResult<&str, RowParsed<W>> {
    let (i, state) = num(i)?;
    let (i, _) = tab(i)?;
    let (i, _) = tag("Infinity")(i)?;
    Ok((i, RowParsed::InfinityFinalState(state as StateId)))
}

fn row_parsed<W: SerializableSemiring>(i: &str) -> IResult<&str, RowParsed<W>> {
    alt((transition, infinity_final_state, final_state))(i)
}

pub fn vec_rows_parsed<W: SerializableSemiring>(i: &str) -> IResult<&str, Vec<RowParsed<W>>> {
    separated_list(tag("\n"), row_parsed)(i)
}
