use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::StateId;
use anyhow::Result;

/// Trait defining the methods to control allocation for a wFST
pub trait AllocableFst<W: Semiring>: Fst<W> {
    /// Reserve capacity for at least additional more trs leaving the state.
    /// The FST may reserve more space to avoid frequent allocation.
    /// After calling `reserve_trs`, the capacity will be greater or equal to `num_trs` + `additionnal`
    /// This method has no effects if the capacity is already sufficient
    fn reserve_trs(&mut self, source: StateId, additional: usize) -> Result<()>;

    /// # Safety
    ///
    /// Unsafe behaviour if `state` is not present in Fst.
    ///
    unsafe fn reserve_trs_unchecked(&mut self, state: StateId, additional: usize);

    /// Reserve capacity for at least additional states.
    /// The FST may reserve more space to avoid frequent allocation.
    /// After calling `reserve_states`, the capacity will be greater or equal to `num_states` + `additionnal`
    /// This method has no effects if the capacity is already sufficient
    fn reserve_states(&mut self, additional: usize);

    /// Shrinks the capacity of the states and their leaving trs as much as possible.
    /// It will drop down as close as possible to the number of states and leaving trs.
    fn shrink_to_fit(&mut self);

    /// Shrinks the capacity of the states.
    /// It will drop down as close as possible to the number of states.
    fn shrink_to_fit_states(&mut self);

    /// Shrinks the capacity of the leaving trs for the given state as much as possible.
    /// It will drop down as close as possible to theleaving trs.
    fn shrink_to_fit_trs(&mut self, source: StateId) -> Result<()>;

    /// Shrinks the capacity of the leaving trs for the given state as much as possible.
    /// It will drop down as close as possible to theleaving trs.
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `state` is not present in Fst.
    ///
    unsafe fn shrink_to_fit_trs_unchecked(&mut self, state: StateId);

    /// Returns the number of states the FST can hold without reallocating.
    fn states_capacity(&self) -> usize;
    /// Returns the number of trs for a given state the FST can hold without reallocating.
    fn trs_capacity(&self, source: StateId) -> Result<usize>;

    /// Returns the number of trs for a given state the FST can hold without reallocating.
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `state` is not present in Fst.
    ///
    unsafe fn trs_capacity_unchecked(&self, state: StateId) -> usize;
}
