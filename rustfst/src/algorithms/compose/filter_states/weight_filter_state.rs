use self::super::FilterState;
use crate::parsers::nom_utils::NomCustomError;
use crate::parsers::SerializeBinary;
use anyhow::Result;
use nom::IResult;
use std::io::Write;

use crate::semirings::{Semiring, SerializableSemiring};

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

impl<T: SerializableSemiring> SerializeBinary for WeightFilterState<T> {
    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>> {
        let (i, state) = T::parse_binary(i)?;
        Ok((i, Self { state }))
    }
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.state.write_binary(writer)?;
        Ok(())
    }
}
