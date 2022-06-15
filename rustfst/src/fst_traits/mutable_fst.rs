use anyhow::Result;
use std::cmp::Ordering;
use std::sync::Arc;

use crate::algorithms::closure::ClosureType;
use crate::algorithms::TrMapper;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{ExpandedFst, Fst};
use crate::semirings::Semiring;
use crate::tr::Tr;
use crate::trs_iter_mut::TrsIterMut;
use crate::{Label, StateId, SymbolTable};

/// Trait defining the methods to modify a wFST.
pub trait MutableFst<W: Semiring>: ExpandedFst<W> {
    /// Creates an empty wFST.
    fn new() -> Self;

    /// The state with identifier `state_id` is now the start state.
    /// Note that only one start state is allowed in this implementation. Calling this function twice
    /// will mean losing the first start state.
    /// If the `state_id` doesn't exist an error is raised.
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
    /// assert_eq!(fst.start(), None);
    ///
    /// fst.set_start(s1);
    /// assert_eq!(fst.start(), Some(s1));
    ///
    /// fst.set_start(s2);
    /// assert_eq!(fst.start(), Some(s2));
    /// ```
    fn set_start(&mut self, state_id: StateId) -> Result<()>;

    /// The state with identifier `state_id` is now the start state.
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `state` is not present in Fst.
    ///
    unsafe fn set_start_unchecked(&mut self, state: StateId);

    /// The state with identifier `state_id` is now a final state with a weight `final_weight`.
    /// If the `state_id` doesn't exist an error is raised.
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
    /// assert_eq!(fst.final_weight(s1).unwrap(), None);
    /// assert_eq!(fst.final_weight(s2).unwrap(), None);
    ///
    /// fst.set_final(s1, BooleanWeight::one());
    /// assert_eq!(fst.final_weight(s1).unwrap(), Some(BooleanWeight::one()));
    /// assert_eq!(fst.final_weight(s2).unwrap(), None);
    ///
    /// fst.set_final(s2, BooleanWeight::one());
    /// assert_eq!(fst.final_weight(s1).unwrap(), Some(BooleanWeight::one()));
    /// assert_eq!(fst.final_weight(s2).unwrap(), Some(BooleanWeight::one()));
    /// ```
    fn set_final<S: Into<W>>(&mut self, state_id: StateId, final_weight: S) -> Result<()>;

    /// Set the final weight of the state with state if `state_id`.
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `state` is not present in Fst.
    ///
    unsafe fn set_final_unchecked<S: Into<W>>(&mut self, state_id: StateId, final_weight: S);

    /// Adds a new state to the current FST. The identifier of the new state is returned
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{BooleanWeight, Semiring};
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
    fn add_state(&mut self) -> StateId;

    /// Add `n` states to the Fst.
    fn add_states(&mut self, n: usize);

    /// Return a mutable iterator on the `Tr`s of the state `state`.
    fn tr_iter_mut(&mut self, state: StateId) -> Result<TrsIterMut<W>>;
    /// Return a mutable iterator on the `Tr`s of the state `state`.
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `state` is not present in Fst.
    ///
    unsafe fn tr_iter_unchecked_mut(&mut self, state_id: StateId) -> TrsIterMut<W>;

    /// Removes a state from an FST. It also removes all the trs starting from another state and
    /// reaching this state. An error is raised if the state `state_id` doesn't exist.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst, StateIterator};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{BooleanWeight, Semiring};
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    ///
    /// assert_eq!(fst.states_iter().count(), 0);
    ///
    /// let s1 = fst.add_state();
    ///
    /// assert_eq!(fst.states_iter().count(), 1);
    ///
    /// fst.del_state(s1);
    ///
    /// assert_eq!(fst.states_iter().count(), 0);
    ///
    /// ```
    fn del_state(&mut self, state_id: StateId) -> Result<()>;

    // TODO: Need to define a correct behaviour is the same state is present multiple times in the iterator
    /// Removes multiple states from an FST. If one of the states doesn't exist, an error is raised.
    ///
    /// # Warning
    ///
    /// This method modifies the id of the states that are left in the FST. Id that were used before
    /// calling this function should no longer be used.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst, StateIterator};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{BooleanWeight, Semiring};
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    ///
    /// assert_eq!(fst.states_iter().count(), 0);
    ///
    /// let s1 = fst.add_state();
    /// let s2 = fst.add_state();
    ///
    /// assert_eq!(fst.states_iter().count(), 2);
    ///
    /// let states_to_remove = vec![s1, s2];
    /// fst.del_states(states_to_remove.into_iter());
    ///
    /// assert_eq!(fst.states_iter().count(), 0);
    ///
    /// ```
    fn del_states<T: IntoIterator<Item = StateId>>(&mut self, states: T) -> Result<()>;

    /// Remove all the states in the FST. As a result, all the trs are also removed,
    /// as well as the start state and all the fina states.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst, StateIterator};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{BooleanWeight, Semiring};
    /// let mut fst = VectorFst::<BooleanWeight>::new();
    ///
    /// assert_eq!(fst.states_iter().count(), 0);
    ///
    /// let s1 = fst.add_state();
    /// let s2 = fst.add_state();
    ///
    /// assert_eq!(fst.states_iter().count(), 2);
    ///
    /// fst.del_all_states();
    ///
    /// assert_eq!(fst.states_iter().count(), 0);
    ///
    /// ```
    fn del_all_states(&mut self);

    /// Remove transitions from the fst at state `state`. Transitions are specified with
    /// their index. The `to_del` vector MUST be sorted.
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `state` is not present in Fst.
    ///
    unsafe fn del_trs_id_sorted_unchecked(&mut self, state: StateId, to_del: &[usize]);

    /// Adds a transition to the FST. The transition will start in the state `source`.
    ///
    /// # Errors
    ///
    /// An error is raised if the state `source` doesn't exist.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{ProbabilityWeight, Semiring};
    /// # use rustfst::Tr;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// let mut fst = VectorFst::<ProbabilityWeight>::new();
    /// let s1 = fst.add_state();
    /// let s2 = fst.add_state();
    ///
    /// assert_eq!(fst.num_trs(s1)?, 0);
    /// fst.add_tr(s1, Tr::new(3, 5, 1.2, s2));
    /// assert_eq!(fst.num_trs(s1)?, 1);
    /// # Ok(())
    /// # }
    /// ```
    fn add_tr(&mut self, source: StateId, tr: Tr<W>) -> Result<()>;

    /// Adds a transition to the FST. The transition will start in the state `state`.
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `state` is not present in Fst.
    ///
    unsafe fn add_tr_unchecked(&mut self, state: StateId, tr: Tr<W>);

    /// Adds a transition to the FST. The transition will start in the state `source`.
    ///
    /// # Errors
    ///
    /// An error is raised if the state `source` doesn't exist.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{Semiring, ProbabilityWeight};
    /// # use rustfst::Tr;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// let mut fst = VectorFst::<ProbabilityWeight>::new();
    /// let s1 = fst.add_state();
    /// let s2 = fst.add_state();
    ///
    /// assert_eq!(fst.num_trs(s1)?, 0);
    /// fst.emplace_tr(s1, 3, 5, 1.2, s2);
    /// assert_eq!(fst.num_trs(s1)?, 1);
    /// # Ok(())
    /// # }
    /// ```
    fn emplace_tr<S: Into<W>>(
        &mut self,
        source: StateId,
        ilabel: Label,
        olabel: Label,
        weight: S,
        nextstate: StateId,
    ) -> Result<()> {
        self.add_tr(source, Tr::new(ilabel, olabel, weight, nextstate))
    }

    /// Adds a transition to the FST. The transition will start in the state `state`.
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `state` is not present in Fst.
    ///
    unsafe fn emplace_tr_unchecked<S: Into<W>>(
        &mut self,
        state: StateId,
        ilabel: Label,
        olabel: Label,
        weight: S,
        nextstate: StateId,
    ) {
        self.add_tr_unchecked(state, Tr::new(ilabel, olabel, weight, nextstate))
    }

    /// Set all the `Tr`s leaving the state `state` to the parameters `trs` erasing
    /// the `Tr`s previously stored.
    ///
    /// Be careful as this function doesn't update the `FstProperties`.
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `state` is not present in Fst.
    ///
    unsafe fn set_trs_unchecked(&mut self, source: StateId, trs: Vec<Tr<W>>);

    /// Remove the final weight of a specific state.
    fn delete_final_weight(&mut self, source: StateId) -> Result<()>;

    /// Remove the final weight of state `state`.
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `state` is not present in Fst.
    ///
    unsafe fn delete_final_weight_unchecked(&mut self, source: StateId);

    /// Deletes all the trs leaving a state.
    fn delete_trs(&mut self, source: StateId) -> Result<()>;

    /// Remove all trs leaving a state and return them.
    fn pop_trs(&mut self, source: StateId) -> Result<Vec<Tr<W>>>;

    /// Remove all the `Tr` leaving the state `state` and return them.
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `state` is not present in Fst.
    ///
    unsafe fn pop_trs_unchecked(&mut self, source: StateId) -> Vec<Tr<W>>;

    /// Takes the final weight out of the fst, leaving a None in its place.
    ///
    /// # Errors
    ///
    /// An error is raised if the state with id `state_id` doesn't exist.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{Semiring, ProbabilityWeight};
    /// # use rustfst::Tr;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// let mut fst = VectorFst::<ProbabilityWeight>::new();
    /// let s1 = fst.add_state();
    /// fst.set_final(s1, 1.2)?;
    ///
    /// assert_eq!(fst.final_weight(s1)?, Some(ProbabilityWeight::new(1.2)));
    /// let weight = fst.take_final_weight(s1)?;
    /// assert_eq!(weight, Some(ProbabilityWeight::new(1.2)));
    /// assert_eq!(fst.final_weight(s1)?, None);
    /// # Ok(())
    /// # }
    /// ```
    fn take_final_weight(&mut self, state_id: StateId) -> Result<Option<W>>;

    /// Takes the final weight out of the fst, leaving a None in its place.
    /// This version leads to `undefined behaviour` if the state doesn't exist.
    ///
    /// # Example
    ///
    /// ```
    /// # use rustfst::fst_traits::{CoreFst, MutableFst, ExpandedFst};
    /// # use rustfst::fst_impls::VectorFst;
    /// # use rustfst::semirings::{Semiring, ProbabilityWeight};
    /// # use rustfst::Tr;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// let mut fst = VectorFst::<ProbabilityWeight>::new();
    /// let s1 = fst.add_state();
    /// fst.set_final(s1, 1.2)?;
    ///
    /// assert_eq!(fst.final_weight(s1)?, Some(ProbabilityWeight::new(1.2)));
    /// let weight = unsafe {fst.take_final_weight_unchecked(s1)};
    /// assert_eq!(weight, Some(ProbabilityWeight::new(1.2)));
    /// assert_eq!(fst.final_weight(s1)?, None);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `state` is not present in Fst.
    ///
    unsafe fn take_final_weight_unchecked(&mut self, state: StateId) -> Option<W>;

    fn sort_trs_unchecked<F: Fn(&Tr<W>, &Tr<W>) -> Ordering>(&mut self, state: StateId, f: F);

    /// Remove duplicate Trs leaving the state `state` with the same `ilabel`, `olabel`, `weight`
    /// and `nextstate`.
    ///
    /// Be careful as this function doesn't update the FstProperties!
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `state` not present in Fst.
    ///
    unsafe fn unique_trs_unchecked(&mut self, state: StateId);

    /// Merge the Trs leaving the state `state` with the same `ilabel`, `olabel`
    /// and `nextstate` and sum their weights.
    ///
    /// Be careful as this function doesn't update the FstProperties!
    ///
    /// # Safety
    ///
    /// Unsafe behaviour if `state` not present in Fst.
    ///
    unsafe fn sum_trs_unchecked(&mut self, state: StateId);

    /// This operation computes the concatenative closure.
    /// If A transduces string `x` to `y` with weight `a`,
    /// then the closure transduces `x` to `y` with weight `a`,
    /// `xx` to `yy` with weight `a ⊗ a`, `xxx` to `yyy` with weight `a ⊗ a ⊗ a`, etc.
    ///  If closure_star then the empty string is transduced to itself with weight `1` as well.
    fn closure(&mut self, closure_type: ClosureType) {
        crate::algorithms::closure::closure(self, closure_type)
    }

    /// Maps a transition using a `TrMapper` object.
    fn tr_map<M: TrMapper<W>>(&mut self, mapper: &mut M) -> Result<()> {
        crate::algorithms::tr_map(self, mapper)
    }

    /// Set the internal properties of the Fst. All the set properties must be verified by the Fst!
    fn set_properties(&mut self, props: FstProperties);

    /// Set only a subset of the internal properties of the Fst.
    fn set_properties_with_mask(&mut self, props: FstProperties, mask: FstProperties);

    /// Compute the properties verified by the Fst (with a mask) and update
    /// the internal property bits.
    fn compute_and_update_properties(&mut self, mask: FstProperties) -> Result<FstProperties> {
        let mut knownprops = FstProperties::empty();
        let testprops =
            crate::fst_properties::compute_fst_properties(self, mask, &mut knownprops, true)?;
        self.set_properties_with_mask(testprops, knownprops);
        Ok(testprops & mask)
    }

    /// Compute all the properties verified by the Fst and update the internal property bits.
    fn compute_and_update_properties_all(&mut self) -> Result<FstProperties> {
        self.compute_and_update_properties(FstProperties::all_properties())
    }

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

    /// Destructively relabel the Fst with new Symbol Tables.
    ///
    /// Relabelling refers to the operation where all the labels of an Fst are mapped to the equivalent labels
    /// of a new `SymbolTable`.
    /// If the Fst has a label `1` corresponding to the symbol "alpha" in the current symbol table and "alpha"
    /// is mapped to 4 in a new SymbolTable, then all the 1 are going to be mapped to 4.
    ///
    /// Parameters :
    /// - old_isymbols: Input `SymbolTable` used to build the Fst. If `None`, uses the Input `SymbolTable` attached to the Fst.
    /// - new_isymbols: New Input `SymbolTable` to use.
    /// - attach_new_isymbols: Whether to attach the new Input `SymbolTable` to the Fst. If False, the resulting Fst won't contain any attached Input `SymbolTable`.
    /// - old_osymbols: Output `SymbolTable` used to build the Fst. If `None`, uses the Output `SymbolTable` attached to the Fst
    /// - new_osymbols: New Output `SymbolTable` to use.
    /// - attach_new_osymbols: Whether to attach the new Output `SymbolTable` to the Fst. If False, the resulting Fst won't contain any attached Output `SymbolTable`.
    fn relabel_tables(
        &mut self,
        old_isymbols: Option<&Arc<SymbolTable>>,
        new_isymbols: &Arc<SymbolTable>,
        attach_new_isymbols: bool,
        old_osymbols: Option<&Arc<SymbolTable>>,
        new_osymbols: &Arc<SymbolTable>,
        attach_new_osymbols: bool,
    ) -> Result<()> {
        let old_isymbols = if let Some(s) = old_isymbols {
            s
        } else {
            self.input_symbols()
                .ok_or_else(|| format_err!("No input SymbolTable found"))?
        };
        let old_isymbols = Arc::clone(old_isymbols);

        let old_osymbols = if let Some(s) = old_osymbols {
            s
        } else {
            self.output_symbols()
                .ok_or_else(|| format_err!("No output SymbolTable found"))?
        };
        let old_osymbols = Arc::clone(old_osymbols);

        for s in 0..self.num_states() {
            let mut trs = self.tr_iter_mut(s as StateId)?;
            for i in 0..trs.len() {
                let tr = trs.get(i).unwrap();

                let old_ilabel = tr.ilabel;
                let isymbol = old_isymbols.get_symbol(old_ilabel).ok_or_else(|| {
                    format_err!("Old Input SymbolTable doesn't contain label {}", old_ilabel)
                })?;
                let new_ilabel = new_isymbols.get_label(isymbol).ok_or_else(|| {
                    format_err!("New Input SymbolTable doesn't contain symbol {:?}", isymbol)
                })?;

                let old_olabel = tr.olabel;
                let osymbol = old_osymbols.get_symbol(old_olabel).ok_or_else(|| {
                    format_err!(
                        "Old Output SymbolTable doesn't contain label {}",
                        old_olabel
                    )
                })?;
                let new_olabel = new_osymbols.get_label(osymbol).ok_or_else(|| {
                    format_err!(
                        "New Output SymbolTable doesn't contain symbol {:?}",
                        osymbol
                    )
                })?;

                trs.set_ilabel(i, new_ilabel)?;
                trs.set_olabel(i, new_olabel)?;
            }
        }

        self.take_input_symbols();
        self.take_output_symbols();

        if attach_new_isymbols {
            self.set_input_symbols(Arc::clone(new_isymbols));
        }

        if attach_new_osymbols {
            self.set_output_symbols(Arc::clone(new_osymbols));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fst;
    use crate::prelude::{TropicalWeight, VectorFst};
    use crate::symt;
    use crate::utils::transducer;

    #[test]
    fn test_relabel_tables() -> Result<()> {
        let fst: VectorFst<TropicalWeight> = fst![1, 2, 3 => 3, 1, 1; 0.1];

        let old_isymt = Arc::new(symt!["a", "b", "c"]);
        let new_isymt = Arc::new(symt!["b", "c", "a"]);

        let old_osymt = Arc::new(symt!["aa", "bb", "cc"]);
        let new_osymt = Arc::new(symt!["cc", "bb", "aa"]);

        let fst_ref: VectorFst<TropicalWeight> = fst![3, 1, 2 => 1, 3, 3; 0.1];

        let mut fst_2 = fst.clone();
        fst_2.relabel_tables(
            Some(&old_isymt),
            &new_isymt,
            true,
            Some(&old_osymt),
            &new_osymt,
            true,
        )?;
        assert_eq!(fst_2, fst_ref);
        assert_eq!(fst_2.input_symbols(), Some(&new_isymt));
        assert_eq!(fst_2.output_symbols(), Some(&new_osymt));

        let mut fst_3 = fst.clone();
        fst_3.relabel_tables(
            Some(&old_isymt),
            &new_isymt,
            false,
            Some(&old_osymt),
            &new_osymt,
            false,
        )?;
        assert_eq!(fst_3, fst_ref);
        assert_eq!(fst_3.input_symbols(), None);
        assert_eq!(fst_3.output_symbols(), None);

        let mut fst_4 = fst.clone();
        fst_4.set_input_symbols(Arc::clone(&old_isymt));
        fst_4.set_output_symbols(Arc::clone(&old_osymt));
        fst_4.relabel_tables(None, &new_isymt, true, None, &new_osymt, false)?;
        assert_eq!(fst_4, fst_ref);
        assert_eq!(fst_4.input_symbols(), Some(&new_isymt));
        assert_eq!(fst_4.output_symbols(), None);

        let mut fst_5 = fst.clone();
        fst_5.set_input_symbols(Arc::clone(&old_isymt));
        fst_5.set_output_symbols(Arc::clone(&old_osymt));
        fst_5.relabel_tables(None, &new_isymt, false, None, &new_osymt, true)?;
        assert_eq!(fst_5, fst_ref);
        assert_eq!(fst_5.input_symbols(), None);
        assert_eq!(fst_5.output_symbols(), Some(&new_osymt));

        Ok(())
    }
}
