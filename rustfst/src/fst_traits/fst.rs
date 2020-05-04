use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::tr_filters::{InputEpsilonTrFilter, OutputEpsilonTrFilter, TrFilter};
use crate::fst_traits::iterators::{StateIterator, TrIterator};
use crate::fst_traits::FstIterator;
use crate::semirings::Semiring;
use crate::{StateId, SymbolTable};

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
    /// assert_eq!(fst.final_weight(s1).unwrap(), None);
    /// assert_eq!(fst.final_weight(s2).unwrap(), Some(&BooleanWeight::one()));
    /// assert!(fst.final_weight(s2 + 1).is_err());
    /// ```
    fn final_weight(&self, state_id: StateId) -> Result<Option<&<Self as CoreFst>::W>>;
    unsafe fn final_weight_unchecked(&self, state_id: StateId) -> Option<&<Self as CoreFst>::W>;

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
    unsafe fn num_trs_unchecked(&self, s: StateId) -> usize;

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
        let w = self.final_weight(state_id)?;
        Ok(w.is_some())
    }
    #[inline]
    unsafe fn is_final_unchecked(&self, state_id: StateId) -> bool {
        self.final_weight_unchecked(state_id).is_some()
    }

    /// Check whether a state is the start state or not.
    #[inline]
    fn is_start(&self, state_id: StateId) -> bool {
        Some(state_id) == self.start()
    }
}

/// Trait defining the minimum interface necessary for a wFST.
pub trait Fst:
    CoreFst + for<'a> TrIterator<'a> + for<'b> StateIterator<'b> + for<'c> FstIterator<'c> + Debug
{
    // TODO: Move niepsilons and noepsilons to required methods.
    /// Returns the number of trs with epsilon input labels leaving a state.
    ///
    /// # Example :
    /// ```
    /// # use rustfst::fst_traits::{MutableFst, Fst};
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
    fn num_input_epsilons(&self, state: StateId) -> Result<usize> {
        let filter = InputEpsilonTrFilter {};
        Ok(self.tr_iter(state)?.filter(|v| filter.keep(v)).count())
    }

    /// Returns the number of trs with epsilon output labels leaving a state.
    ///
    /// # Example :
    /// ```
    /// # use rustfst::fst_traits::{MutableFst, Fst};
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
    fn num_output_epsilons(&self, state: StateId) -> Result<usize> {
        let filter = OutputEpsilonTrFilter {};
        Ok(self.tr_iter(state)?.filter(|v| filter.keep(v)).count())
    }

    /// Returns true if the Fst is an acceptor. False otherwise.
    /// Acceptor means for all transition, transition.ilabel == transition.olabel
    fn is_acceptor(&self) -> bool {
        let states: Vec<_> = self.states_iter().collect();
        for state in states {
            for tr in self.tr_iter(state).unwrap() {
                if tr.ilabel != tr.olabel {
                    return false;
                }
            }
        }
        true
    }

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

    fn set_symts_from_fst<OF: Fst>(&mut self, other_fst: &OF) {
        if let Some(symt) = other_fst.input_symbols() {
            self.set_input_symbols(Arc::clone(symt));
        } else {
            self.take_input_symbols();
        }

        if let Some(symt) = other_fst.output_symbols() {
            self.set_output_symbols(Arc::clone(symt));
        } else {
            self.take_output_symbols();
        }
    }
}
