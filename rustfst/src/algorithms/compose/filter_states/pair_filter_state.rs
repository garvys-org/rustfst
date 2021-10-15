use self::super::FilterState;
use crate::parsers::nom_utils::NomCustomError;
use crate::parsers::SerializeBinary;
use anyhow::Result;
use nom::IResult;
use std::io::Write;

/// Filter state that is the combination of two filter states.
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct PairFilterState<FS1, FS2> {
    state: (FS1, FS2),
}

impl<FS1: FilterState, FS2: FilterState> FilterState for PairFilterState<FS1, FS2> {
    type Type = (FS1, FS2);

    fn new(value: Self::Type) -> Self {
        Self { state: value }
    }

    fn new_no_state() -> Self {
        Self {
            state: (FS1::new_no_state(), FS2::new_no_state()),
        }
    }

    fn state(&self) -> &Self::Type {
        &self.state
    }
}

impl<FS1: FilterState + SerializeBinary, FS2: FilterState + SerializeBinary> SerializeBinary
    for PairFilterState<FS1, FS2>
{
    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>> {
        let (i, fs1) = FS1::parse_binary(i)?;
        let (i, fs2) = FS2::parse_binary(i)?;
        Ok((i, Self { state: (fs1, fs2) }))
    }
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.state.0.write_binary(writer)?;
        self.state.1.write_binary(writer)?;
        Ok(())
    }
}

impl<FS1, FS2> PairFilterState<FS1, FS2> {
    pub fn state1(&self) -> &FS1 {
        &self.state.0
    }

    pub fn state2(&self) -> &FS2 {
        &self.state.1
    }
}
