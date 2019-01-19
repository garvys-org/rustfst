use failure::Fallible;

use crate::arc::Arc;
use crate::semirings::Semiring;
use crate::{StateId, EPS_LABEL};
use std::fmt::Display;

/// Trait defining necessary methods for a wFST to access start states and final states.
pub trait CoreFst {
    /// Weight use in the wFST. This type must implement the Semiring trait.
    type W: Semiring;

    /// Returns the ID of the start state of the wFST if it exists else none.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::BooleanWeight;
    /// // 1 - Create an FST
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    /// let s = fst.add_state();
    /// fst.set_start(s);
    ///
    /// // 2 - Access the start state
    /// let start_state = fst.start();
    /// assert_eq!(start_state, Some(s));
    /// ```
    fn start(&self) -> Option<StateId>;

    /// Retrieves the final weight of a state (if the state is a final one).
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{BooleanWeight, Semiring};
    /// // 1 - Create an FST
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    /// let s1 = fst.add_state();
    /// let s2 = fst.add_state();
    /// fst.set_final(s2, BooleanWeight::one());
    ///
    /// // 2 - Access the final weight of each state
    /// assert_eq!(fst.final_weight(s1), None);
    /// assert_eq!(fst.final_weight(s2), Some(BooleanWeight::one()));
    /// ```
    fn final_weight(&self, state_id: StateId) -> Option<<Self as CoreFst>::W>;

    /// Number of arcs leaving a specific state in the wFST.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{BooleanWeight, Semiring};
    /// # use rustfst::Arc;
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    /// let s1 = fst.add_state();
    /// let s2 = fst.add_state();
    ///
    /// assert_eq!(fst.num_arcs(s1).unwrap(), 0);
    /// fst.add_arc(s1, Arc::new(3, 5, BooleanWeight::new(true), s2));
    /// assert_eq!(fst.num_arcs(s1).unwrap(), 1);
    /// ```
    fn num_arcs(&self, s: StateId) -> Fallible<usize>;

    /// Returns whether or not the state with identifier passed as parameters is a final state.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{BooleanWeight, Semiring};
    /// // 1 - Create an FST
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    /// let s1 = fst.add_state();
    /// let s2 = fst.add_state();
    /// fst.set_final(s2, BooleanWeight::one());
    ///
    /// // 2 - Test if a state is final
    /// assert!(!fst.is_final(s1));
    /// assert!(fst.is_final(s2));
    /// ```
    fn is_final(&self, state_id: StateId) -> bool {
        self.final_weight(state_id).is_some()
    }

    /// Check whether a state is the start state or not.
    fn is_start(&self, state_id: StateId) -> bool {
        Some(state_id) == self.start()
    }
}

/// Trait to iterate over the states of a wFST.
pub trait StateIterator<'a> {
    /// Iterator used to iterate over the `state_id` of the states of an FST.
    type Iter: Iterator<Item = StateId> + Clone;

    /// Creates an iterator over the `state_id` of the states of an FST.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst, StateIterator};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{BooleanWeight, Semiring};
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    ///
    /// let s1 = fst.add_state();
    /// let s2 = fst.add_state();
    ///
    /// for state_id in fst.states_iter() {
    ///     println!("State ID : {:?}", state_id);
    /// }
    ///
    /// let states : Vec<_> = fst.states_iter().collect();
    /// assert_eq!(states, vec![s1, s2]);
    /// ```
    fn states_iter(&'a self) -> Self::Iter;
}

/// Trait to iterate over the outgoing arcs of a particular state in a wFST
pub trait ArcIterator<'a>: CoreFst
where
    Self::W: 'a,
{
    /// Iterator used to iterate over the arcs leaving a state of an FST.
    type Iter: Iterator<Item = &'a Arc<Self::W>> + Clone;

    fn arcs_iter(&'a self, state_id: StateId) -> Fallible<Self::Iter>;
}

/// Trait defining the minimum interface necessary for a wFST.
pub trait Fst:
    CoreFst + PartialEq + Clone + for<'a> ArcIterator<'a> + for<'b> StateIterator<'b> + Display
{
    /// Returns the number of arcs with epsilon input labels leaving a state.
    ///
    /// # Example :
    /// ```
    /// # use rustfst::fst_traits::{MutableFst, Fst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{Semiring, IntegerWeight};
    /// # use rustfst::EPS_LABEL;
    /// # use rustfst::Arc;
    /// let mut fst = VectorFst::new();
    /// let s0 = fst.add_state();
    /// let s1 = fst.add_state();
    ///
    /// fst.add_arc(s0, Arc::new(EPS_LABEL, 18, IntegerWeight::one(), s1));
    /// fst.add_arc(s0, Arc::new(76, EPS_LABEL, IntegerWeight::one(), s1));
    /// fst.add_arc(s0, Arc::new(EPS_LABEL, 18, IntegerWeight::one(), s1));
    /// fst.add_arc(s0, Arc::new(45, 18, IntegerWeight::one(), s0));
    /// fst.add_arc(s1, Arc::new(76, 18, IntegerWeight::one(), s1));
    ///
    /// assert_eq!(fst.num_input_epsilons(s0).unwrap(), 2);
    /// assert_eq!(fst.num_input_epsilons(s1).unwrap(), 0);
    /// ```
    fn num_input_epsilons(&self, state: StateId) -> Fallible<usize> {
        Ok(self
            .arcs_iter(state)?
            .filter(|v| v.ilabel == EPS_LABEL)
            .count())
    }

    /// Returns the number of arcs with epsilon output labels leaving a state.
    ///
    /// # Example :
    /// ```
    /// # use rustfst::fst_traits::{MutableFst, Fst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{Semiring, IntegerWeight};
    /// # use rustfst::EPS_LABEL;
    /// # use rustfst::Arc;
    /// let mut fst = VectorFst::new();
    /// let s0 = fst.add_state();
    /// let s1 = fst.add_state();
    ///
    /// fst.add_arc(s0, Arc::new(EPS_LABEL, 18, IntegerWeight::one(), s1));
    /// fst.add_arc(s0, Arc::new(76, EPS_LABEL, IntegerWeight::one(), s1));
    /// fst.add_arc(s0, Arc::new(EPS_LABEL, 18, IntegerWeight::one(), s1));
    /// fst.add_arc(s0, Arc::new(45, 18, IntegerWeight::one(), s0));
    /// fst.add_arc(s1, Arc::new(76, 18, IntegerWeight::one(), s1));
    ///
    /// assert_eq!(fst.num_output_epsilons(s0).unwrap(), 1);
    /// assert_eq!(fst.num_output_epsilons(s1).unwrap(), 0);
    /// ```
    fn num_output_epsilons(&self, state: StateId) -> Fallible<usize> {
        Ok(self
            .arcs_iter(state)?
            .filter(|v| v.olabel == EPS_LABEL)
            .count())
    }
}
