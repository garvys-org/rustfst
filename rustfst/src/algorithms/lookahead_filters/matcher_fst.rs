use crate::algorithms::lookahead_filters::add_on::FstAddOn;
use crate::algorithms::lookahead_matchers::LookaheadMatcher;
use crate::algorithms::matchers::MatchType;
use crate::fst_traits::{ArcIterator, CoreFst, ExpandedFst, Fst, FstIterator, StateIterator};
use crate::SymbolTable;
use failure::Fallible;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

pub struct MatcherFst<F, M, T> {
    fst_add_on: FstAddOn<F, (T, T)>,
    matcher: PhantomData<M>,
}

impl<F: Debug, M, T: Debug> Debug for MatcherFst<F, M, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

impl<F, M, T> MatcherFst<F, M, T> {
    pub fn data(&self, match_type: MatchType) -> &T {
        let data = self.fst_add_on.add_on();
        if match_type == MatchType::MatchInput {
            &data.0
        } else {
            &data.1
        }
    }
}

impl<'fst, F: ExpandedFst + 'fst, M: LookaheadMatcher<'fst, F::W, F = F>>
    MatcherFst<&'fst F, M, M::MatcherData>
{
    pub fn new(fst: &'fst F) -> Fallible<Self> {
        let imatcher = M::new(fst, MatchType::MatchInput)?;
        let omatcher = M::new(fst, MatchType::MatchOutput)?;

        let add_on = (
            imatcher.data().unwrap().clone(),
            omatcher.data().unwrap().clone(),
        );

        let fst_add_on = FstAddOn::new(fst, add_on);

        panic!("Add init -> LookAheadRelabeler");

        Ok(Self {
            fst_add_on,
            matcher: PhantomData,
        })
    }

    pub fn init_matcher(&self, match_type: MatchType) -> Fallible<M> {
        M::new_with_data(
            self.fst_add_on.fst(),
            match_type,
            Some(self.data(match_type).clone()),
        )
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

impl<F: Fst, M, T: Debug> Fst for MatcherFst<F, M, T>
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
