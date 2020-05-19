pub use determinize_fsa::DeterminizeFsa;
pub(self) use determinize_fsa_op::DeterminizeFsaOp;
pub use determinize_static::{determinize, determinize_with_distance};
pub(self) use divisors::{DefaultCommonDivisor, GallicCommonDivisor};
pub(self) use element::{DeterminizeElement, DeterminizeStateTuple, DeterminizeTr, WeightedSubset};
pub(self) use state_table::DeterminizeStateTable;

mod determinize_fsa;
mod determinize_fsa_op;
mod determinize_static;
mod divisors;
mod element;
mod state_table;

/// Determinization type.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum DeterminizeType {
    /// Input transducer is known to be functional (or error).
    DeterminizeFunctional,
    /// Input transducer is NOT known to be functional.
    DeterminizeNonFunctional,
    /// Input transducer is not known to be functional but only keep the min of
    /// of ambiguous outputs.
    DeterminizeDisambiguate,
}
