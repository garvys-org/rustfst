use nom::bytes::complete::take_while;
use nom::character::complete::digit1;
use nom::combinator::map_res;
use nom::IResult;
use std::str::FromStr;

use nom::error::ErrorKind;
use nom::error::{FromExternalError, ParseError};

#[derive(Debug, PartialEq)]
pub enum NomCustomError<I> {
    SymbolTableError(String),
    Nom(I, ErrorKind),
}

impl<I> ParseError<I> for NomCustomError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        NomCustomError::Nom(input, kind)
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I, E> FromExternalError<I, E> for NomCustomError<I> {
    fn from_external_error(input: I, kind: ErrorKind, _e: E) -> Self {
        NomCustomError::Nom(input, kind)
    }
}

pub fn num<V: FromStr>(i: &str) -> IResult<&str, V> {
    map_res(digit1, |s: &str| s.parse())(i)
}

pub fn word(i: &str) -> IResult<&str, String> {
    let (i, letters) = take_while(|c: char| (c != ' ') && (c != '\t') && (c != '\n'))(i)?;
    Ok((i, letters.to_string()))
}
