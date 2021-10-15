use self::super::FilterState;
use crate::parsers::nom_utils::NomCustomError;
use crate::parsers::{parse_bin_u8, write_bin_u8, SerializeBinary};
use anyhow::Result;
use nom::IResult;
use std::io::Write;

/// Single non-blocking filter state.
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct TrivialFilterState {
    state: bool,
}

impl FilterState for TrivialFilterState {
    type Type = bool;

    fn new(value: Self::Type) -> Self {
        Self { state: value }
    }

    fn new_no_state() -> Self {
        Self::new(false)
    }

    fn state(&self) -> &Self::Type {
        &self.state
    }
}

impl SerializeBinary for TrivialFilterState {
    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>> {
        let (i, state) = parse_bin_u8(i)?;
        if state == 1 {
            Ok((i, Self { state: true }))
        } else {
            Ok((i, Self { state: false }))
        }
    }
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<()> {
        if self.state {
            write_bin_u8(writer, 1)?;
        } else {
            write_bin_u8(writer, 0)?;
        }
        Ok(())
    }
}
