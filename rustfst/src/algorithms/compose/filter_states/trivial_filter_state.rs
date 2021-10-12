use self::super::{FilterState, SerializableFilterState};
use crate::parsers::nom_utils::NomCustomError;
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

impl SerializableFilterState for TrivialFilterState {
    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>> {
        unimplemented!()
    }
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<()> {
        unimplemented!()
    }
}
