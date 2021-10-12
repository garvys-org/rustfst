pub use integer_filter_state::IntegerFilterState;
pub use list_filter_state::ListFilterState;
pub use pair_filter_state::PairFilterState;
use std::fmt::Debug;
use std::hash::Hash;
pub use trivial_filter_state::TrivialFilterState;
pub use weight_filter_state::WeightFilterState;

use crate::parsers::nom_utils::NomCustomError;
use anyhow::Result;
use nom::IResult;
use std::io::Write;

/// The filter state interface represents the state of a (e.g., composition) filter.
pub trait FilterState: PartialEq + Clone + Eq + Hash + Debug {
    type Type;

    fn new(value: Self::Type) -> Self;
    fn new_no_state() -> Self;
    fn state(&self) -> &Self::Type;
}

pub trait SerializableFilterState: Sized {
    /// Parse a filter state from a binary buffer.
    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>>;
    /// Writes a filter state to a writable buffer.
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<()>;
}

mod integer_filter_state;
mod list_filter_state;
mod pair_filter_state;
mod trivial_filter_state;
mod weight_filter_state;
