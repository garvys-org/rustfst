pub use integer_filter_state::{
    CharFilterState, IntFilterState, IntegerFilterState, ShortFilterState,
};
pub use list_filter_state::ListFilterState;
pub use pair_filter_state::PairFilterState;
pub use trivial_filter_state::TrivialFilterState;
pub use weight_filter_state::WeightFilterState;

/// The filter state interface represents the state of a (e.g., composition) filter.
pub trait FilterState: Default + PartialEq {
    type Type;

    fn new(value: Self::Type) -> Self;
    fn state(&self) -> Option<&Self::Type>;
}

mod integer_filter_state;
mod list_filter_state;
mod pair_filter_state;
mod trivial_filter_state;
mod weight_filter_state;
