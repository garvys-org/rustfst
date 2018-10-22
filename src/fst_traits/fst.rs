use arc::Arc;
use semirings::Semiring;
use std::fmt::Display;
use Result;
use StateId;

/// Trait defining the minimum interface necessary for a wFST
pub trait Fst:
    CoreFst + PartialEq + Clone + for<'a> ArcIterator<'a> + for<'b> StateIterator<'b> + Display
{
}

/// Trait defining necessary methods for a wFST to access start states and final states
pub trait CoreFst {
    /// Weight use in the wFST. This type must implement the Semiring trait
    type W: Semiring;

    /// Returns the ID of the start state of the wFST if it exists else none.
    ///
    /// # Example
    ///
    /// ```
    /// use rustfst::fst_traits::{CoreFst, MutableFst};
    /// use rustfst::fst_impls::VectorFst;
    /// use rustfst::semirings::BooleanWeight;
    ///
    /// // 1 - Create an FST
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    /// let s = fst.add_state();
    /// fst.set_start(&s);
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
    /// use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// use rustfst::fst_impls::VectorFst;
    /// use rustfst::semirings::{BooleanWeight, Semiring};
    ///
    /// // 1 - Create an FST
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    /// let s1 = fst.add_state();
    /// let s2 = fst.add_state();
    /// fst.set_final(&s2, BooleanWeight::one());
    ///
    /// // 2 - Access the final weight of each state
    /// assert_eq!(fst.final_weight(&s1), None);
    /// assert_eq!(fst.final_weight(&s2), Some(BooleanWeight::one()));
    /// ```
    fn final_weight(&self, &StateId) -> Option<<Self as CoreFst>::W>;

    /// Total number of arcs in the wFST. This is the sum of the outgoing arcs of each state.
    ///
    /// # Example
    ///
    /// ```
    /// use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// use rustfst::fst_impls::VectorFst;
    /// use rustfst::semirings::{BooleanWeight, Semiring};
    /// use rustfst::arc::Arc;
    ///
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    /// let s1 = fst.add_state();
    /// let s2 = fst.add_state();
    ///
    /// assert_eq!(fst.num_arcs(), 0);
    /// fst.add_arc(&s1, Arc::new(3, 5, BooleanWeight::new(true), s2));
    /// assert_eq!(fst.num_arcs(), 1);
    /// ```
    fn num_arcs(&self) -> usize;

    /// Returns whether or not the state with identifier passed as parameters is a final state.
    ///
    /// # Example
    ///
    /// ```
    /// use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// use rustfst::fst_impls::VectorFst;
    /// use rustfst::semirings::{BooleanWeight, Semiring};
    ///
    /// // 1 - Create an FST
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    /// let s1 = fst.add_state();
    /// let s2 = fst.add_state();
    /// fst.set_final(&s2, BooleanWeight::one());
    ///
    /// // 2 - Test if a state is final
    /// assert!(!fst.is_final(&s1));
    /// assert!(fst.is_final(&s2));
    /// ```
    fn is_final(&self, state_id: &StateId) -> bool {
        self.final_weight(state_id).is_some()
    }

    //type Symtab: IntoIterator<Item=String>;
    //fn get_isyms(&self) -> Option<Self::Symtab>;
    //fn get_osyms(&self) -> Option<Self::Symtab>;
}

/// Trait to iterate over the states of a wFST
pub trait StateIterator<'a> {
    /// Iterator used to iterate over the `state_id` of the states of an FST.
    type Iter: Iterator<Item = StateId> + Clone;

    /// Creates an iterator over the `state_id` of the states of an FST.
    ///
    /// # Example
    ///
    /// ```
    /// use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst, StateIterator};
    /// use rustfst::fst_impls::VectorFst;
    /// use rustfst::semirings::{BooleanWeight, Semiring};
    ///
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

/// Trait to iterate over the outgoing arcs of a partical state in a wFST
pub trait ArcIterator<'a>: CoreFst
where
    Self::W: 'a,
{
    /// Iterator used to iterate over the arcs leaving a state of an FST.
    type Iter: Iterator<Item = &'a Arc<Self::W>> + Clone;

    fn arcs_iter(&'a self, &StateId) -> Result<Self::Iter>;
}

macro_rules! add_or_fst {
    ($semiring:tt, $fst_type:ty) => {
        impl<$semiring: 'static + Semiring> Add for $fst_type {
            type Output = Result<$fst_type>;

            fn add(self, rhs: $fst_type) -> Self::Output {
                concat(&self, &rhs)
            }
        }

        impl<$semiring: 'static + Semiring> BitOr for $fst_type {
            type Output = Result<$fst_type>;

            fn bitor(self, rhs: $fst_type) -> Self::Output {
                union(&self, &rhs)
            }
        }
    };
}

macro_rules! display_single_state {
    ($fst:expr, $state_id:expr, $f: expr) => {
        for arc in $fst.arcs_iter($state_id).unwrap() {
            write!(
                $f,
                "{}\t{}\t{}\t{}\t{}\n",
                $state_id, &arc.nextstate, &arc.ilabel, &arc.olabel, &arc.weight
            )?;
        }
    };
}

macro_rules! display_fst {
    ($semiring:tt, $fst_type:ty) => {
        impl<$semiring: 'static + Semiring> fmt::Display for $fst_type
        where
            $semiring::Type: fmt::Display,
        {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                if let Some(start_state) = self.start() {
                    // Firstly print the arcs leaving the start state
                    display_single_state!(self, &start_state, f);

                    // Secondly, print the arcs leaving all the other states
                    for state_id in self.states_iter() {
                        if state_id != start_state {
                            display_single_state!(self, &state_id, f);
                        }
                    }

                    // Finally, print the final states with their weight
                    for final_state in self.final_states_iter() {
                        write!(
                            f,
                            "{}\t{}\n",
                            &final_state.state_id, &final_state.final_weight
                        );
                    }
                }
                Ok(())
            }
        }
    };
}
