mod closure_fst;
mod closure_static;

pub use closure_fst::ClosureFst;
pub use closure_static::closure;

/// Defines the different types of closure : Star or Plus.
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum ClosureType {
    ClosureStar,
    ClosurePlus,
}
