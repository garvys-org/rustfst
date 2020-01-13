use std::collections::HashMap;
use std::rc::Rc;

use failure::{format_err, Fallible};

use crate::algorithms::ReplaceFst;
use crate::arc::Arc;
use crate::fst_traits::{
    AllocableFst, ArcIterator, CoreFst, ExpandedFst, FinalStatesIterator, Fst, MutableFst,
    StateIterator,
};
use crate::semirings::Semiring;
use crate::{StateId, SymbolTable};

/// Performs the union of two wFSTs. If A transduces string `x` to `y` with weight `a`
/// and `B` transduces string `w` to `v` with weight `b`, then their union transduces `x` to `y`
/// with weight `a` and `w` to `v` with weight `b`.
///
/// # Example
/// ```
/// # #[macro_use] extern crate rustfst;
/// # use failure::Fallible;
/// # use rustfst::utils::transducer;
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::fst_traits::PathsIterator;
/// # use rustfst::FstPath;
/// # use rustfst::algorithms::union;
/// # use std::collections::HashSet;
/// # fn main() -> Fallible<()> {
/// let fst_a : VectorFst<IntegerWeight> = fst![2 => 3];
/// let fst_b : VectorFst<IntegerWeight> = fst![6 => 5];
///
/// let fst_res : VectorFst<IntegerWeight> = union(&fst_a, &fst_b)?;
/// let paths : HashSet<_> = fst_res.paths_iter().collect();
///
/// let mut paths_ref = HashSet::<FstPath<IntegerWeight>>::new();
/// paths_ref.insert(fst_path![2 => 3]);
/// paths_ref.insert(fst_path![6 => 5]);
///
/// assert_eq!(paths, paths_ref);
/// # Ok(())
/// # }
/// ```
pub fn union<W, F1, F2, F3>(fst_1: &F1, fst_2: &F2) -> Fallible<F3>
where
    W: Semiring,
    F1: ExpandedFst<W = W>,
    F2: ExpandedFst<W = W>,
    F3: MutableFst<W = W>,
{
    let mut fst_out = F3::new();

    let start_state = fst_out.add_state();
    fst_out.set_start(start_state)?;

    let mapping_states_fst_1 = fst_out.add_fst(fst_1)?;
    let mapping_states_fst_2 = fst_out.add_fst(fst_2)?;

    add_epsilon_arc_to_initial_state(fst_1, &mapping_states_fst_1, &mut fst_out)?;
    add_epsilon_arc_to_initial_state(fst_2, &mapping_states_fst_2, &mut fst_out)?;

    set_new_final_states(fst_1, &mapping_states_fst_1, &mut fst_out)?;
    set_new_final_states(fst_2, &mapping_states_fst_2, &mut fst_out)?;

    Ok(fst_out)
}

fn add_epsilon_arc_to_initial_state<F1, F2>(
    fst: &F1,
    mapping: &HashMap<StateId, StateId>,
    fst_out: &mut F2,
) -> Fallible<()>
where
    F1: ExpandedFst,
    F2: MutableFst,
{
    let start_state = fst_out.start().unwrap();
    if let Some(old_start_state_fst) = fst.start() {
        fst_out.add_arc(
            start_state,
            Arc::new(
                0,
                0,
                <F2 as CoreFst>::W::one(),
                *mapping.get(&old_start_state_fst).unwrap(),
            ),
        )?;
    }
    Ok(())
}

fn set_new_final_states<W, F1, F2>(
    fst: &F1,
    mapping: &HashMap<StateId, StateId>,
    fst_out: &mut F2,
) -> Fallible<()>
where
    W: Semiring,
    F1: ExpandedFst<W = W>,
    F2: MutableFst<W = W>,
{
    for old_final_state in fst.final_states_iter() {
        let final_state = mapping.get(&old_final_state.state_id).ok_or_else(|| {
            format_err!(
                "Key {:?} doesn't exist in mapping",
                old_final_state.state_id
            )
        })?;
        fst_out.set_final(*final_state, old_final_state.final_weight.clone())?;
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnionFst<F: Fst + 'static>(ReplaceFst<F, F>)
where
    F::W: 'static;

impl<F: Fst + MutableFst + AllocableFst> UnionFst<F>
where
    F::W: 'static,
{
    pub fn new(fst1: F, fst2: F) -> Fallible<Self> {
        let mut rfst = F::new();
        rfst.add_states(2);
        rfst.set_start(0)?;
        unsafe { rfst.set_final_unchecked(1, F::W::one()) };
        if let Some(isymt) = fst1.input_symbols() {
            rfst.set_input_symbols(isymt);
        }
        if let Some(osymt) = fst1.output_symbols() {
            rfst.set_output_symbols(osymt);
        }
        unsafe { rfst.reserve_arcs_unchecked(0, 2) };
        unsafe { rfst.add_arc_unchecked(0, Arc::new(0, std::usize::MAX, F::W::one(), 1)) };
        unsafe { rfst.add_arc_unchecked(0, Arc::new(0, std::usize::MAX - 1, F::W::one(), 1)) };

        let mut fst_tuples = Vec::with_capacity(3);
        fst_tuples.push((0, rfst));
        fst_tuples.push((std::usize::MAX, fst1));
        fst_tuples.push((std::usize::MAX - 1, fst2));

        Ok(UnionFst(ReplaceFst::new(fst_tuples, 0, false)?))
    }
}

impl<F: Fst> CoreFst for UnionFst<F>
where
    F::W: 'static,
{
    type W = F::W;

    fn start(&self) -> Option<usize> {
        self.0.start()
    }

    fn final_weight(&self, state_id: usize) -> Fallible<Option<&Self::W>> {
        self.0.final_weight(state_id)
    }

    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<&Self::W> {
        self.0.final_weight_unchecked(state_id)
    }

    fn num_arcs(&self, s: usize) -> Fallible<usize> {
        self.0.num_arcs(s)
    }

    unsafe fn num_arcs_unchecked(&self, s: usize) -> usize {
        self.0.num_arcs_unchecked(s)
    }
}

impl<'a, F: Fst + 'static> StateIterator<'a> for UnionFst<F>
where
    F::W: 'static,
{
    type Iter = <ReplaceFst<F, F> as StateIterator<'a>>::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.0.states_iter()
    }
}

impl<'a, F: Fst + 'static> ArcIterator<'a> for UnionFst<F>
where
    F::W: 'static,
{
    type Iter = <ReplaceFst<F, F> as ArcIterator<'a>>::Iter;

    fn arcs_iter(&'a self, state_id: usize) -> Fallible<Self::Iter> {
        self.0.arcs_iter(state_id)
    }

    unsafe fn arcs_iter_unchecked(&'a self, state_id: usize) -> Self::Iter {
        self.0.arcs_iter_unchecked(state_id)
    }
}

impl<F: Fst + 'static> Fst for UnionFst<F>
where
    F::W: 'static,
{
    fn input_symbols(&self) -> Option<Rc<SymbolTable>> {
        self.0.input_symbols()
    }

    fn output_symbols(&self) -> Option<Rc<SymbolTable>> {
        self.0.output_symbols()
    }
}
