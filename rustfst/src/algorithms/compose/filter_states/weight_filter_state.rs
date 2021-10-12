use self::super::{FilterState, SerializableFilterState};
use crate::parsers::nom_utils::NomCustomError;
use anyhow::Result;
use nom::IResult;
use std::io::Write;

use crate::semirings::Semiring;

/// Filter state that is a weight implementing the Semiring trait.
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct WeightFilterState<W> {
    state: W,
}

impl<W: Semiring> FilterState for WeightFilterState<W> {
    type Type = W;

    fn new(value: Self::Type) -> Self {
        Self { state: value }
    }

    fn new_no_state() -> Self {
        Self { state: W::zero() }
    }

    fn state(&self) -> &Self::Type {
        &self.state
    }
}

impl<T: Semiring> SerializableFilterState for WeightFilterState<T> {
    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>> {
        unimplemented!()
    }
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<()> {
        unimplemented!()
    }
}
