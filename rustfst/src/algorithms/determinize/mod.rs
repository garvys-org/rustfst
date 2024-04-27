use determinize_fsa::DeterminizeFsa;
use determinize_fsa_op::DeterminizeFsaOp;
pub use determinize_static::{
    determinize, determinize_with_config, determinize_with_distance, DeterminizeConfig,
};
use divisors::{DefaultCommonDivisor, GallicCommonDivisor};
use element::{DeterminizeElement, DeterminizeStateTuple, DeterminizeTr, WeightedSubset};
use state_table::DeterminizeStateTable;

mod determinize_fsa;
mod determinize_fsa_op;
mod determinize_static;
mod divisors;
mod element;
mod state_table;

/// Determinization type.
#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum DeterminizeType {
    /// Input transducer is known to be functional (or error).
    DeterminizeFunctional,
    /// Input transducer is NOT known to be functional.
    DeterminizeNonFunctional,
    /// Input transducer is not known to be functional but only keep the min of
    /// of ambiguous outputs.
    DeterminizeDisambiguate,
}
