use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::tr_filters::{InputEpsilonTrFilter, OutputEpsilonTrFilter, TrFilter};
use crate::fst_traits::iterators::StateIterator;
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
    fn final_weight(&self, state_id: StateId) -> Result<Option<W>>;
    unsafe fn final_weight_unchecked(&self, state_id: StateId) -> Option<W>;

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
    fn num_trs(&self, s: StateId) -> Result<usize> {
        Ok(self.get_trs(s)?.len())
    }
    unsafe fn num_trs_unchecked(&self, s: StateId) -> usize {
        self.get_trs_unchecked(s).len()
    }

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

    fn get_trs(&self, state_id: StateId) -> Result<Self::TRS>;
    unsafe fn get_trs_unchecked(&self, state_id: StateId) -> Self::TRS;
}

/// Trait defining the minimum interface necessary for a wFST.
pub trait Fst<W: Semiring>:
    CoreFst<W> + for<'b> StateIterator<'b> + Debug + for<'c> FstIterator<'c, W>
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
        Ok(self
            .get_trs(state)?
            .iter()
            .filter(|v| filter.keep(v))
            .count())
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
        Ok(self
            .get_trs(state)?
            .iter()
            .filter(|v| filter.keep(v))
            .count())
    }

    /// Returns true if the Fst is an acceptor. False otherwise.
    /// Acceptor means for all transition, transition.ilabel == transition.olabel
    fn is_acceptor(&self) -> bool {
        let states: Vec<_> = self.states_iter().collect();
        unsafe {
            for state in states {
                for tr in self.get_trs_unchecked(state).trs() {
                    if tr.ilabel != tr.olabel {
                        return false;
                    }
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

    fn set_symts_from_fst<W2: Semiring, OF: Fst<W2>>(&mut self, other_fst: &OF) {
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
}

pub struct FinalStatesIterator<'a, W, F>
where
    W: Semiring,
    F: Fst<W>,
{
    fst: &'a F,
    state_iter: <F as StateIterator<'a>>::Iter,
    w: PhantomData<W>,
}

impl<'a, W, F> Iterator for FinalStatesIterator<'a, W, F>
where
    W: Semiring,
    F: Fst<W>,
{
    type Item = StateId;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(s) = self.state_iter.next() {
                if unsafe { self.fst.is_final_unchecked(s) } {
                    return Some(s);
                }
            } else {
                return None;
            }
        }
    }
}
