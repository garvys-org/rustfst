use arc::Arc;
use semirings::Semiring;
use StateId;

/// Trait defining the minimum interface necessary for a wFST
pub trait Fst: CoreFst + PartialEq + for<'a> ArcIterator<'a> + for<'b> StateIterator<'b> {}

/// Trait defining necessary methods for a wFST to access start states and final states
pub trait CoreFst {
    /// Weight use in the wFST. This type must implement the Semiring trait
    type W: Semiring;

    /// Returns the ID of the start state of the wFST
    ///
    /// # Example
    ///
    /// ```
    /// use rustfst::fst_traits::MutableFst;
    /// use rustfst::fst_impls::VectorFst;
    /// use rustfst::semirings::BooleanWeight;
    /// use rustfst::fst_traits::CoreFst;
    ///
    /// // Create an FST
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    /// let s = fst.add_state();
    /// fst.set_start(&s);
    /// 
    /// // Access start state
    /// let start_state = fst.start();
    /// assert_eq!(start_state, Some(s));
    /// ```
    fn start(&self) -> Option<StateId>;

    /// Retrieve the final weight of a state (if the state is a final one)
    fn final_weight(&self, &StateId) -> Option<<Self as CoreFst>::W>;

    /// Total number of arcs in the wFST. This is the sum of the outgoing arcs of each state
    fn num_arcs(&self) -> usize;

    /// Whether the state with identifier passed as parameters is a final state
    fn is_final(&self, state_id: &StateId) -> bool {
        self.final_weight(state_id).is_some()
    }

    //type Symtab: IntoIterator<Item=String>;
    //fn get_isyms(&self) -> Option<Self::Symtab>;
    //fn get_osyms(&self) -> Option<Self::Symtab>;
}

/// Trait to iterate over the states of a wFST
pub trait StateIterator<'a> {
    type Iter: Iterator<Item = StateId>;
    fn states_iter(&'a self) -> Self::Iter;
}

/// Trait to iterate over the outgoing arcs of a partical state in a wFST
pub trait ArcIterator<'a>: CoreFst
where
    Self::W: 'a,
{
    type Iter: Iterator<Item = &'a Arc<Self::W>>;
    fn arcs_iter(&'a self, &StateId) -> Self::Iter;
}
