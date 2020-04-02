use std::cell::RefCell;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

use failure::Fallible;

use crate::algorithms::compose::lookahead_matchers::{LabelLookAheadRelabeler, LookaheadMatcher};
use crate::algorithms::compose::matchers::MatchType;
use crate::algorithms::compose::FstAddOn;
use crate::algorithms::compose::LabelReachableData;
use crate::fst_traits::{
    ArcIterator, CoreFst, ExpandedFst, Fst, FstIntoIterator, FstIterator, MutableFst, StateIterator,
};
use crate::SymbolTable;

#[derive(Clone, PartialEq, Debug)]
pub struct MatcherFst<F, M, T> {
    fst_add_on: FstAddOn<F, (Option<Rc<RefCell<T>>>, Option<Rc<RefCell<T>>>)>,
    matcher: PhantomData<M>,
}

impl<F, M, T> MatcherFst<F, M, T> {
    pub fn fst(&self) -> &F {
        self.fst_add_on.fst()
    }

    pub fn addon(&self) -> &(Option<Rc<RefCell<T>>>, Option<Rc<RefCell<T>>>) {
        self.fst_add_on.add_on()
    }

    pub fn data(&self, match_type: MatchType) -> Option<&Rc<RefCell<T>>> {
        let data = self.fst_add_on.add_on();
        if match_type == MatchType::MatchInput {
            data.0.as_ref()
        } else {
            data.1.as_ref()
        }
    }
}

// TODO: To be generalized
impl<
        'a,
        'fst: 'a,
        F: MutableFst + 'fst,
        M: LookaheadMatcher<'a, F::W, F = F, MatcherData = LabelReachableData>,
    > MatcherFst<F, M, M::MatcherData>
{
    pub fn new(fst: F) -> Fallible<Self> {
        let imatcher_data = M::create_data(&fst, MatchType::MatchInput)?;
        let omatcher_data = M::create_data(&fst, MatchType::MatchOutput)?;

        let add_on = (imatcher_data, omatcher_data);
        let mut fst_add_on = FstAddOn::new(fst, add_on);
        LabelLookAheadRelabeler::init(&mut fst_add_on)?;
        Ok(Self {
            fst_add_on,
            matcher: PhantomData,
        })
    }
}

impl<F: CoreFst, M, T> CoreFst for MatcherFst<F, M, T> {
    type W = F::W;

    fn start(&self) -> Option<usize> {
        self.fst_add_on.start()
    }

    fn final_weight(&self, state_id: usize) -> Fallible<Option<&Self::W>> {
        self.fst_add_on.final_weight(state_id)
    }

    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<&Self::W> {
        self.fst_add_on.final_weight_unchecked(state_id)
    }

    fn num_arcs(&self, s: usize) -> Fallible<usize> {
        self.fst_add_on.num_arcs(s)
    }

    unsafe fn num_arcs_unchecked(&self, s: usize) -> usize {
        self.fst_add_on.num_arcs_unchecked(s)
    }
}

impl<'a, F: StateIterator<'a>, M, T> StateIterator<'a> for MatcherFst<F, M, T> {
    type Iter = <F as StateIterator<'a>>::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.fst_add_on.states_iter()
    }
}

impl<'a, F: ArcIterator<'a>, M, T> ArcIterator<'a> for MatcherFst<F, M, T>
where
    F::W: 'a,
{
    type Iter = <F as ArcIterator<'a>>::Iter;

    fn arcs_iter(&'a self, state_id: usize) -> Fallible<Self::Iter> {
        self.fst_add_on.arcs_iter(state_id)
    }

    unsafe fn arcs_iter_unchecked(&'a self, state_id: usize) -> Self::Iter {
        self.fst_add_on.arcs_iter_unchecked(state_id)
    }
}

impl<'a, F: FstIterator<'a>, M, T> FstIterator<'a> for MatcherFst<F, M, T>
where
    F::W: 'a,
{
    type ArcsIter = F::ArcsIter;
    type FstIter = F::FstIter;

    fn fst_iter(&'a self) -> Self::FstIter {
        self.fst_add_on.fst_iter()
    }
}

impl<F: Fst, M: Debug, T: Debug> Fst for MatcherFst<F, M, T>
where
    F::W: 'static,
{
    fn input_symbols(&self) -> Option<Rc<SymbolTable>> {
        self.fst_add_on.input_symbols()
    }

    fn output_symbols(&self) -> Option<Rc<SymbolTable>> {
        self.fst_add_on.output_symbols()
    }

    fn set_input_symbols(&mut self, symt: Rc<SymbolTable>) {
        self.fst_add_on.set_input_symbols(symt)
    }

    fn set_output_symbols(&mut self, symt: Rc<SymbolTable>) {
        self.fst_add_on.set_output_symbols(symt)
    }

    fn unset_input_symbols(&mut self) -> Option<Rc<SymbolTable>> {
        self.fst_add_on.unset_input_symbols()
    }

    fn unset_output_symbols(&mut self) -> Option<Rc<SymbolTable>> {
        self.fst_add_on.unset_output_symbols()
    }
}

impl<F: ExpandedFst, M: Debug + Clone + PartialEq, T: Debug + Clone + PartialEq> ExpandedFst
    for MatcherFst<F, M, T>
where
    F::W: 'static,
{
    fn num_states(&self) -> usize {
        self.fst_add_on.num_states()
    }
}

impl<F: FstIntoIterator, M, T: Debug> FstIntoIterator for MatcherFst<F, M, T>
where
    F::W: 'static,
{
    type ArcsIter = F::ArcsIter;
    type FstIter = F::FstIter;

    fn fst_into_iter(self) -> Self::FstIter {
        self.fst_add_on.fst_into_iter()
    }
}
