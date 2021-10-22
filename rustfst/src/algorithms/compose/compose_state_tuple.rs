use crate::algorithms::compose::filter_states::FilterState;
use crate::parsers::nom_utils::NomCustomError;
use crate::parsers::{parse_bin_u64, write_bin_u64, SerializeBinary};
use crate::StateId;

use anyhow::Result;
use nom::IResult;
use std::io::Write;

#[derive(Default, PartialEq, Eq, Clone, Hash, PartialOrd, Debug)]
pub struct ComposeStateTuple<FS: FilterState> {
    pub fs: FS,
    pub s1: StateId,
    pub s2: StateId,
}

impl<FS: FilterState + SerializeBinary> SerializeBinary for ComposeStateTuple<FS> {
    /// Parse a filter state from a binary buffer.
    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>> {
        let (i, fs) = FS::parse_binary(i)?;
        let (i, s1) = parse_bin_u64(i)?;
        let (i, s2) = parse_bin_u64(i)?;
        Ok((
            i,
            Self {
                fs,
                s1: s1 as StateId,
                s2: s2 as StateId,
            },
        ))
    }
    /// Writes a filter state to a writable buffer.
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.fs.write_binary(writer)?;
        write_bin_u64(writer, self.s1 as u64)?;
        write_bin_u64(writer, self.s2 as u64)?;
        Ok(())
    }
}
