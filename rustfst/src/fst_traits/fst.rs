use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;

use crate::fst_properties::FstProperties;
use crate::fst_traits::final_states_iterator::FinalStatesIterator;
use crate::fst_traits::iterators::StateIterator;
use crate::fst_traits::paths_iterator::PathsIterator;
use crate::fst_traits::string_paths_iterator::StringPathsIterator;
use crate::fst_traits::FstIterator;
use crate::semirings::Semiring;
use crate::trs::Trs;
use crate::{StateId, SymbolTable};

/// Trait defining necessary methods for a wFST to access start states and final states.
pub trait CoreFst<W: Semiring> {
    /// Weight use in the wFST. This type must implement the Semiring trait.
    type TRS: Trs<W>;

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
    /// assert_eq!(fst.final_weight(s1).unwrap(), None);
    /// assert_eq!(fst.final_weight(s2).unwrap(), Some(BooleanWeight::one()));
    /// assert!(fst.final_weight(s2 + 1).is_err());
    /// ```
    fn final_weight(&self, state: StateId) -> Result<Option<W>>;

    /// Retrieves the final weight of a state (if the state is a final one).
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `state` is not present in Fst.
    ///
    unsafe fn final_weight_unchecked(&self, state: StateId) -> Option<W>;

    /// Number of trs leaving a specific state in the wFST.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{BooleanWeight, Semiring};
    /// # use rustfst::Tr;
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    /// let s1 = fst.add_state();
    /// let s2 = fst.add_state();
    ///
    /// assert_eq!(fst.num_trs(s1).unwrap(), 0);
    /// fst.add_tr(s1, Tr::new(3, 5, BooleanWeight::new(true), s2));
    /// assert_eq!(fst.num_trs(s1).unwrap(), 1);
    /// ```
    fn num_trs(&self, s: StateId) -> Result<usize>;

    /// Number of trs leaving a specific state in the wFST.
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `state` is not present in Fst.
    ///
    unsafe fn num_trs_unchecked(&self, state: StateId) -> usize;

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
    /// assert_eq!(fst.is_final(s1).unwrap(), false);
    /// assert_eq!(fst.is_final(s2).unwrap(), true);
    /// assert!(fst.is_final(s2 + 1).is_err());
    /// ```
    #[inline]
    fn is_final(&self, state_id: StateId) -> Result<bool> {
        Ok(self
            .final_weight(state_id)?
            .map(|final_weight| final_weight != W::zero())
            .unwrap_or(false))
    }

    /// Returns whether or not the state with identifier passed as parameters is a final state.
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `state` is not present in Fst.
    ///
    #[inline]
    unsafe fn is_final_unchecked(&self, state: StateId) -> bool {
        self.final_weight_unchecked(state).is_some()
    }

    /// Check whether a state is the start state or not.
    #[inline]
    fn is_start(&self, state_id: StateId) -> bool {
        Some(state_id) == self.start()
    }

    /// Get an iterator on the transitions leaving state `state`.
    fn get_trs(&self, state_id: StateId) -> Result<Self::TRS>;

    /// Get an iterator on the transitions leaving state `state`.
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `state` is not present in Fst.
    ///
    unsafe fn get_trs_unchecked(&self, state: StateId) -> Self::TRS;

    /// Retrieve the `FstProperties` stored in the Fst. As a result, all the properties returned
    /// are verified by the Fst but some other properties might be true as well despite the flag
    /// not being set.
    fn properties(&self) -> FstProperties;

    /// Apply a mask to the `FstProperties` returned.
    fn properties_with_mask(&self, mask: FstProperties) -> FstProperties {
        self.properties() & mask
    }

    /// Retrieve the `FstProperties` in the Fst and check that all the
    /// properties in `props_known` are known (not the same as true). If not an error is returned.
    ///
    /// A property is known if we known for sure if it is true of false.
    fn properties_check(&self, props_known: FstProperties) -> Result<FstProperties> {
        let props = self.properties();
        if !props.knows(props_known) {
            bail!(
                "Properties are not known : {:?}. Properties of the Fst : {:?}",
                props_known,
                props
            )
        }
        Ok(props)
    }

    /// Returns the number of trs with epsilon input labels leaving a state.
    ///
    /// # Example :
    /// ```
    /// # use rustfst::fst_traits::{MutableFst, Fst, CoreFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{Semiring, IntegerWeight};
    /// # use rustfst::EPS_LABEL;
    /// # use rustfst::Tr;
    /// let mut fst = VectorFst::<IntegerWeight>::new();
    /// let s0 = fst.add_state();
    /// let s1 = fst.add_state();
    ///
    /// fst.add_tr(s0, Tr::new(EPS_LABEL, 18, IntegerWeight::one(), s1));
    /// fst.add_tr(s0, Tr::new(76, EPS_LABEL, IntegerWeight::one(), s1));
    /// fst.add_tr(s0, Tr::new(EPS_LABEL, 18, IntegerWeight::one(), s1));
    /// fst.add_tr(s0, Tr::new(45, 18, IntegerWeight::one(), s0));
    /// fst.add_tr(s1, Tr::new(76, 18, IntegerWeight::one(), s1));
    ///
    /// assert_eq!(fst.num_input_epsilons(s0).unwrap(), 2);
    /// assert_eq!(fst.num_input_epsilons(s1).unwrap(), 0);
    /// ```
    fn num_input_epsilons(&self, state: StateId) -> Result<usize>;

    /// Returns the number of trs with epsilon output labels leaving a state.
    ///
    /// # Example :
    /// ```
    /// # use rustfst::fst_traits::{MutableFst, Fst, CoreFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{Semiring, IntegerWeight};
    /// # use rustfst::EPS_LABEL;
    /// # use rustfst::Tr;
    /// let mut fst = VectorFst::<IntegerWeight>::new();
    /// let s0 = fst.add_state();
    /// let s1 = fst.add_state();
    ///
    /// fst.add_tr(s0, Tr::new(EPS_LABEL, 18, IntegerWeight::one(), s1));
    /// fst.add_tr(s0, Tr::new(76, EPS_LABEL, IntegerWeight::one(), s1));
    /// fst.add_tr(s0, Tr::new(EPS_LABEL, 18, IntegerWeight::one(), s1));
    /// fst.add_tr(s0, Tr::new(45, 18, IntegerWeight::one(), s0));
    /// fst.add_tr(s1, Tr::new(76, 18, IntegerWeight::one(), s1));
    ///
    /// assert_eq!(fst.num_output_epsilons(s0).unwrap(), 1);
    /// assert_eq!(fst.num_output_epsilons(s1).unwrap(), 0);
    /// ```
    fn num_output_epsilons(&self, state: StateId) -> Result<usize>;
}

/// Trait defining the minimum interface necessary for a wFST.
pub trait Fst<W: Semiring>:
    CoreFst<W> + for<'b> StateIterator<'b> + Debug + for<'c> FstIterator<'c, W>
{
    /// Retrieves the input `SymbolTable` associated to the Fst.
    /// If no SymbolTable has been previously attached then `None` is returned.
    fn input_symbols(&self) -> Option<&Arc<SymbolTable>>;

    /// Retrieves the output `SymbolTable` associated to the Fst.
    /// If no SymbolTable has been previously attached then `None` is returned.
    fn output_symbols(&self) -> Option<&Arc<SymbolTable>>;

    /// Attaches an output `SymbolTable` to the Fst.
    /// The `SymbolTable` is not duplicated with the use of Arc.
    fn set_input_symbols(&mut self, symt: Arc<SymbolTable>);

    /// Attaches an output `SymbolTable` to the Fst.
    /// The `SymbolTable` is not duplicated with the use of Arc.
    fn set_output_symbols(&mut self, symt: Arc<SymbolTable>);

    /// Removes the input symbol table from the Fst and retrieves it.
    fn take_input_symbols(&mut self) -> Option<Arc<SymbolTable>>;
    /// Removes the output symbol table from the Fst and retrieves it.
    fn take_output_symbols(&mut self) -> Option<Arc<SymbolTable>>;

    /// Returns an Iterator on the final states along with their weight.
    fn final_states_iter(&self) -> FinalStatesIterator<W, Self>
    where
        Self: std::marker::Sized,
    {
        FinalStatesIterator {
            fst: self,
            state_iter: self.states_iter(),
            w: PhantomData,
        }
    }

    /// Returns an Iterator on the paths accepted by the Fst.
    ///
    /// # Example :
    /// ```
    /// # use std::sync::Arc;
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::TropicalWeight;
    /// # use rustfst::{Semiring, SymbolTable, symt};
    /// # use rustfst::utils::transducer;
    /// # use rustfst::fst_traits::Fst;
    /// let mut fst : VectorFst<_> = transducer(&[1, 2, 3], &[4, 5], TropicalWeight::one());
    ///
    /// let paths : Vec<_> = fst.paths_iter().collect();
    /// assert_eq!(paths.len(), 1);
    /// assert_eq!(paths[0].ilabels.as_slice(), &[1, 2, 3]);
    /// assert_eq!(paths[0].olabels.as_slice(), &[4, 5]);
    /// assert_eq!(&paths[0].weight, &TropicalWeight::one());
    /// ```
    fn paths_iter(&self) -> PathsIterator<W, Self>
    where
        Self: std::marker::Sized,
    {
        PathsIterator::new(self)
    }

    /// Returns an Iterator on the paths accepted by the Fst. Plus, handles the SymbolTable
    /// allowing to retrieve the strings instead of only the sequence of labels.
    ///
    /// # Example :
    /// ```
    /// # use std::sync::Arc;
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::TropicalWeight;
    /// # use rustfst::{Semiring, SymbolTable, symt};
    /// # use rustfst::utils::transducer;
    /// # use rustfst::fst_traits::Fst;
    /// let mut fst : VectorFst<_> = transducer(&[1, 2, 3], &[4, 5], TropicalWeight::one());
    /// let symt = symt!["a", "b", "c", "d", "e"];
    /// let symt = Arc::new(symt);
    /// fst.set_input_symbols(Arc::clone(&symt));
    /// fst.set_output_symbols(Arc::clone(&symt));
    ///
    /// let paths : Vec<_> = fst.string_paths_iter().unwrap().collect();
    /// assert_eq!(paths.len(), 1);
    /// assert_eq!(paths[0].ilabels(), &[1, 2, 3]);
    /// assert_eq!(paths[0].olabels(), &[4, 5]);
    /// assert_eq!(paths[0].weight(), &TropicalWeight::one());
    /// assert_eq!(paths[0].istring().unwrap(), "a b c".to_string());
    /// assert_eq!(paths[0].ostring().unwrap(), "d e".to_string());
    /// ```
    fn string_paths_iter(&self) -> Result<StringPathsIterator<W, Self>>
    where
        Self: std::marker::Sized,
    {
        StringPathsIterator::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fst_traits::MutableFst;
    use crate::prelude::TropicalWeight;
    use crate::prelude::VectorFst;

    #[test]
    fn test_is_final() -> Result<()> {
        let mut fst = VectorFst::<TropicalWeight>::new();
        let s = fst.add_state();
        assert!(!fst.is_final(s)?);
        fst.set_final(s, TropicalWeight::zero())?;
        assert!(!fst.is_final(s)?);
        fst.set_final(s, TropicalWeight::one())?;
        assert!(fst.is_final(s)?);
        Ok(())
    }
}
