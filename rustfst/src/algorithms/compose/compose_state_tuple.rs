use crate::algorithms::compose::filter_states::{FilterState, SerializableFilterState};
use crate::StateId;
use crate::parsers::nom_utils::NomCustomError;
use anyhow::Result;
use nom::IResult;
use std::io::Write;

#[derive(Default, PartialEq, Eq, Clone, Hash, PartialOrd, Debug)]
pub struct ComposeStateTuple<FS: FilterState + SerializableFilterState> {
    pub fs: FS,
    pub s1: StateId,
    pub s2: StateId,
}

impl<FS: FilterState + SerializableFilterState> ComposeStateTuple<FS> {
        /// Parse a filter state from a binary buffer.
        fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>> {
            unimplemented!()
        }
        /// Writes a filter state to a writable buffer.
        fn write_binary<W: Write>(&self, writer: &mut W) -> Result<()> {
            unimplemented!()
        }
}