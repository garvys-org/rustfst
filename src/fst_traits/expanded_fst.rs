use fst_traits::Fst;

pub trait ExpandedFst: Fst {
    /// Returns the number of states that contains the FST. They are all counted even if some states
    /// are not on a successful path (doesn't perform triming).
    ///
    /// # Example
    ///
    /// ```
    /// use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// use rustfst::fst_impls::VectorFst;
    /// use rustfst::semirings::{BooleanWeight, Semiring};
    ///
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
}
