use std::fmt::Debug;
use std::hash::Hash;

use self::super::FilterState;
use crate::parsers::nom_utils::NomCustomError;
use crate::parsers::{parse_bin_u64, write_bin_u64, SerializeBinary};
use anyhow::Result;
use nom::multi::count;
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

impl SerializeBinary for ListFilterState {
    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>> {
        let (i, num_states) = parse_bin_u64(i)?;
        let (i, state) = count(parse_bin_u64, num_states as usize)(i)?;
        Ok((
            i,
            Self {
                state: state.into_iter().map(|it| it as usize).collect(),
            },
        ))
    }
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<()> {
        let num_states = self.state.len();
        write_bin_u64(writer, num_states as u64)?;
        for state in self.state.iter() {
            write_bin_u64(writer, *state as u64)?
        }
        Ok(())
    }
}
