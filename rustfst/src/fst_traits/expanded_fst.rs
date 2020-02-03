use failure::Fallible;

use crate::fst_properties::compute_fst_properties;
use crate::fst_properties::FstProperties;
use crate::fst_traits::Fst;

/// Trait defining the necessary methods that should implement an ExpandedFST e.g
/// a FST where all the states are already computed and not computed on the fly.
pub trait ExpandedFst: Fst + Clone + PartialEq {
    /// Returns the number of states that contains the FST. They are all counted even if some states
    /// are not on a successful path (doesn't perform triming).
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{BooleanWeight, Semiring};
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    ///
    /// assert_eq!(fst.num_states(), 0);
    ///
    /// fst.add_state();
    /// assert_eq!(fst.num_states(), 1);
    ///
    /// fst.add_state();
    /// assert_eq!(fst.num_states(), 2);
    ///
    /// ```
    fn num_states(&self) -> usize;

    /// Compute the properties verified by the Fst.
    fn properties(&self) -> Fallible<FstProperties> {
        compute_fst_properties(self)
    }
}
