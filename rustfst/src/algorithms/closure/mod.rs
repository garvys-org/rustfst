mod closure;
mod closure_fst;

pub use closure::closure;
pub use closure_fst::ClosureFst;

/// Defines the different types of closure : Star or Plus.
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum ClosureType {
    ClosureStar,
    ClosurePlus,
}
