use crate::fst_traits::CoreFst;
use crate::semirings::Semiring;
use crate::tr::Tr;
use crate::StateId;

/// Trait to iterate over the states of a wFST.
pub trait StateIterator<'a> {
    /// Iterator used to iterate over the `state_id` of the states of an FST.
    type Iter: Iterator<Item = StateId>;

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

pub struct FstIterData<W, I> {
    pub state_id: StateId,
    pub final_weight: Option<W>,
    pub trs: I,
    pub num_trs: usize,
}

pub trait FstIntoIterator<W: Semiring>: CoreFst<W> {
    type TrsIter: Iterator<Item = Tr<W>>;
    type FstIter: Iterator<Item = FstIterData<W, Self::TrsIter>>;
    fn fst_into_iter(self) -> Self::FstIter;
}

pub trait FstIterator<'a, W: Semiring>: CoreFst<W> {
    type FstIter: Iterator<Item = FstIterData<W, Self::TRS>>;
    fn fst_iter(&'a self) -> Self::FstIter;
}
