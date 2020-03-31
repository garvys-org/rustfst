use self::add_on::FstAddOn;
pub use self::composition::{
    compose, compose_with_config, ComposeConfig, ComposeFilterEnum, ComposeFst,
    ComposeFstImplOptions,
};
pub(crate) use self::interval_reach_visitor::IntervalReachVisitor;
pub(crate) use self::interval_set::{IntInterval, IntervalSet};
pub use self::label_reachable::{LabelReachable, LabelReachableData};
pub use self::matcher_fst::MatcherFst;
pub(crate) use self::state_reachable::StateReachable;

pub mod compose_filters;
mod composition;
pub mod filter_states;
pub mod lookahead_filters;
pub mod lookahead_matchers;
pub mod matchers;

mod add_on;
mod interval_reach_visitor;
mod interval_set;
mod label_reachable;
mod matcher_fst;
mod state_reachable;
