use std::fmt::Debug;
use std::hash::Hash;

use self::super::{FilterState, SerializableFilterState};
use crate::parsers::nom_utils::NomCustomError;
use anyhow::Result;
use nom::IResult;
use std::io::Write;

/// Filter state is a list of signed integer types T. Order matters for equality.
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct ListFilterState {
    state: Vec<usize>,
}

impl FilterState for ListFilterState {
    type Type = Vec<usize>;

    fn new(_value: Self::Type) -> Self {
        unimplemented!()
    }

    fn new_no_state() -> Self {
        unimplemented!()
    }

    fn state(&self) -> &Self::Type {
        unimplemented!()
    }
}

impl SerializableFilterState for ListFilterState {
    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>> {
        unimplemented!()
    }
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<()> {
        unimplemented!()
    }
}
