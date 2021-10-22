pub(crate) mod bin_fst;
pub(crate) mod bin_symt;
pub mod nom_utils;
pub mod text_fst;
pub(crate) mod text_symt;
pub mod utils_parsing;
pub mod utils_serialization;

pub use {nom_utils::NomCustomError, utils_parsing::*, utils_serialization::*};

use anyhow::Result;
use nom::IResult;
use std::io::Write;

pub trait SerializeBinary: Sized {
    /// Parse a struct of type Self from a binary buffer.
    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>>;
    /// Writes a struct to a writable buffer.
    fn write_binary<WB: Write>(&self, writer: &mut WB) -> Result<()>;
}
