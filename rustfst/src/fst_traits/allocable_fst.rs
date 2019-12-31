use failure::Fallible;
use crate::StateId;

/// Trait defining the methods to control allocation for a wFST
pub trait AllocableFst {
    /// Reserve capacity for at least additional more arcs leaving the state.
    /// The FST may reserve more space to avoid frequent allocation. 
    /// After calling `reserve_arcs`, the capacity will be greater or equal to `num_arcs` + `additionnal`
    /// This method has no effects if the capacity is already sufficient
    fn reserve_arcs(&mut self, source: StateId, additional: usize) -> Fallible<()>;
    unsafe fn reserve_arcs_unchecked(&mut self, source: StateId, additional: usize);

    /// Reserve capacity for at least additional states.
    /// The FST may reserve more space to avoid frequent allocation. 
    /// After calling `reserve_states`, the capacity will be greater or equal to `num_states` + `additionnal`
    /// This method has no effects if the capacity is already sufficient
    fn reserve_states(&mut self, additional: usize);


    /// Shrinks the capacity of the states and their leaving arcs as much as possible.
    /// It will drop down as close as possible to the number of states and leaving arcs.
    fn shrink_to_fit(&mut self);

    /// Shrinks the capacity of the states.
    /// It will drop down as close as possible to the number of states.
    fn shrink_to_fit_states(&mut self);

    /// Shrinks the capacity of the leaving arcs for the given state as much as possible.
    /// It will drop down as close as possible to theleaving arcs
    fn shrink_to_fit_arcs(&mut self, source: StateId) -> Fallible<()>;
    unsafe fn shrink_to_fit_arcs_unchecked(&mut self, source: StateId);


    /// Returns the number of states the FST can hold without reallocating.
    fn states_capacity(&self) -> usize;
    /// Returns the number of arcs for a given state the FST can hold without reallocating.
    fn arcs_capacity(&self, source: StateId) -> Fallible<usize>;
    unsafe fn arcs_capacity_unchecked(&self) -> usize;
}