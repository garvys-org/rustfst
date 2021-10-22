use std::fmt::Debug;
use std::hash::Hash;

use crate::{StateId, NO_STATE_ID};

use self::super::FilterState;
use crate::parsers::nom_utils::NomCustomError;
use crate::parsers::{parse_bin_u64, write_bin_u64, SerializeBinary};
use anyhow::Result;
use nom::IResult;
use std::io::Write;

/// Filter state that is a signed integral type.
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct IntegerFilterState {
    state: StateId,
}

impl FilterState for IntegerFilterState {
    type Type = StateId;

    fn new(value: Self::Type) -> Self {
        Self { state: value }
    }
    fn new_no_state() -> Self {
        Self { state: NO_STATE_ID }
    }

    fn state(&self) -> &Self::Type {
        &self.state
    }
}

impl SerializeBinary for IntegerFilterState {
    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>> {
        let (i, state) = parse_bin_u64(i)?;
        Ok((
            i,
            Self {
                state: state as StateId,
            },
        ))
    }
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<()> {
        write_bin_u64(writer, self.state as u64)?;
        Ok(())
    }
}

// pub type IntFilterState = IntegerFilterState<i32>;
// pub type ShortFilterState = IntegerFilterState<i16>;
// pub type CharFilterState = IntegerFilterState<i8>;
