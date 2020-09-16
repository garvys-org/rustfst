use anyhow::Result;

use crate::algorithms::compose::compose_filters::{
    ComposeFilter, ComposeFilterBuilder, SequenceComposeFilterBuilder,
};
use crate::algorithms::compose::matchers::{GenericMatcher, Matcher};
use crate::algorithms::compose::{ComposeFstOp, ComposeFstOpOptions, ComposeStateTuple};
use crate::algorithms::lazy_fst_revamp::{FstCache, LazyFst, StateTable, SimpleVecCache};
use crate::fst_properties::FstProperties;
use crate::fst_traits::{
    AllocableFst, CoreFst, ExpandedFst, Fst, FstIterator, MutableFst, StateIterator,
};
use crate::semirings::Semiring;
use crate::{SymbolTable, TrsVec};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ComposeFst<W: Semiring, CFB: ComposeFilterBuilder<W>, Cache = SimpleVecCache<W>>(
    LazyFst<W, ComposeFstOp<W, CFB>, Cache>,
);

fn create_base<W: Semiring, F1: ExpandedFst<W>, F2: ExpandedFst<W>>(
    fst1: Arc<F1>,
    fst2: Arc<F2>,
) -> Result<
    ComposeFstOp<W, SequenceComposeFilterBuilder<W, GenericMatcher<W, F1>, GenericMatcher<W, F2>>>,
> {
    // TODO: change this once Lookahead matchers are supported.
    let opts = ComposeFstOpOptions::<GenericMatcher<_, _>, GenericMatcher<_, _>, _, _>::default();
    let compose_impl = ComposeFstOp::new(fst1, fst2, opts)?;
    Ok(compose_impl)
}

impl<W: Semiring, CFB: ComposeFilterBuilder<W>, Cache: FstCache<W>> ComposeFst<W, CFB, Cache> {
    pub fn new_with_options(
        fst1: Arc<<<CFB::CF as ComposeFilter<W>>::M1 as Matcher<W>>::F>,
        fst2: Arc<<<CFB::CF as ComposeFilter<W>>::M2 as Matcher<W>>::F>,
        opts: ComposeFstOpOptions<
            CFB::M1,
            CFB::M2,
            CFB,
            StateTable<ComposeStateTuple<<CFB::CF as ComposeFilter<W>>::FS>>,
        >,
    ) -> Result<Self>
    where
        Cache: Default,
    {
        let isymt = fst1.input_symbols().cloned();
        let osymt = fst2.output_symbols().cloned();
        let compose_impl = ComposeFstOp::new(fst1, fst2, opts)?;
        let fst_cache = Cache::default();
        let fst = LazyFst::from_op_and_cache(compose_impl, fst_cache, isymt, osymt);
        Ok(ComposeFst(fst))
    }

    pub fn new_with_options_and_cache(
        fst1: Arc<<<CFB::CF as ComposeFilter<W>>::M1 as Matcher<W>>::F>,
        fst2: Arc<<<CFB::CF as ComposeFilter<W>>::M2 as Matcher<W>>::F>,
        opts: ComposeFstOpOptions<
            CFB::M1,
            CFB::M2,
            CFB,
            StateTable<ComposeStateTuple<<CFB::CF as ComposeFilter<W>>::FS>>,
        >,
        fst_cache: Cache,
    ) -> Result<Self> {
        let isymt = fst1.input_symbols().cloned();
        let osymt = fst2.output_symbols().cloned();
        let compose_impl = ComposeFstOp::new(fst1, fst2, opts)?;
        let fst = LazyFst::from_op_and_cache(compose_impl, fst_cache, isymt, osymt);
        Ok(ComposeFst(fst))
    }

    // TODO: Change API, no really user friendly
    pub fn new(
        fst1: Arc<<<CFB::CF as ComposeFilter<W>>::M1 as Matcher<W>>::F>,
        fst2: Arc<<<CFB::CF as ComposeFilter<W>>::M2 as Matcher<W>>::F>,
    ) -> Result<Self>
    where
        Cache: Default,
    {
        Self::new_with_options(fst1, fst2, ComposeFstOpOptions::default())
    }

    /// Turns the Lazy FST into a static one.
    pub fn compute<F2: MutableFst<W> + AllocableFst<W>>(&self) -> Result<F2> {
        self.0.compute()
    }
}

impl<W: Semiring, F1: ExpandedFst<W>, F2: ExpandedFst<W>>
    ComposeFst<W, SequenceComposeFilterBuilder<W, GenericMatcher<W, F1>, GenericMatcher<W, F2>>>
{
    pub fn new_auto(fst1: Arc<F1>, fst2: Arc<F2>) -> Result<Self> {
        let isymt = fst1.input_symbols().cloned();
        let osymt = fst2.output_symbols().cloned();
        let compose_impl = create_base(fst1, fst2)?;
        let fst_cache = SimpleVecCache::default();
        let fst = LazyFst::from_op_and_cache(compose_impl, fst_cache, isymt, osymt);
        Ok(ComposeFst(fst))
    }
}

impl<W, CFB, Cache> CoreFst<W> for ComposeFst<W, CFB, Cache>
where
    W: Semiring,
    CFB: ComposeFilterBuilder<W>,
    Cache: FstCache<W>,
{
    type TRS = TrsVec<W>;

    fn start(&self) -> Option<usize> {
        self.0.start()
    }

    fn final_weight(&self, state_id: usize) -> Result<Option<W>> {
        self.0.final_weight(state_id)
    }

    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<W> {
        self.0.final_weight_unchecked(state_id)
    }

    fn num_trs(&self, s: usize) -> Result<usize> {
        self.0.num_trs(s)
    }

    unsafe fn num_trs_unchecked(&self, s: usize) -> usize {
        self.0.num_trs_unchecked(s)
    }

    fn get_trs(&self, state_id: usize) -> Result<Self::TRS> {
        self.0.get_trs(state_id)
    }

    unsafe fn get_trs_unchecked(&self, state_id: usize) -> Self::TRS {
        self.0.get_trs_unchecked(state_id)
    }

    fn properties(&self) -> FstProperties {
        self.0.properties()
    }

    fn num_input_epsilons(&self, state: usize) -> Result<usize> {
        self.0.num_input_epsilons(state)
    }

    fn num_output_epsilons(&self, state: usize) -> Result<usize> {
        self.0.num_output_epsilons(state)
    }
}

impl<'a, W, CFB, Cache> StateIterator<'a> for ComposeFst<W, CFB, Cache>
where
    W: Semiring,
    CFB: ComposeFilterBuilder<W> + 'a,
    Cache: FstCache<W> + 'a,
{
    type Iter = <LazyFst<W, ComposeFstOp<W, CFB>, Cache> as StateIterator<'a>>::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.0.states_iter()
    }
}

impl<'a, W, CFB, Cache> FstIterator<'a, W> for ComposeFst<W, CFB, Cache>
where
    W: Semiring,
    CFB: ComposeFilterBuilder<W> + 'a,
    Cache: FstCache<W> + 'a,
{
    type FstIter = <LazyFst<W, ComposeFstOp<W, CFB>, Cache> as FstIterator<'a, W>>::FstIter;

    fn fst_iter(&'a self) -> Self::FstIter {
        self.0.fst_iter()
    }
}

impl<W, CFB, Cache> Fst<W> for ComposeFst<W, CFB, Cache>
where
    W: Semiring,
    CFB: ComposeFilterBuilder<W> + 'static,
    Cache: FstCache<W> + 'static,
{
    fn input_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.0.input_symbols()
    }

    fn output_symbols(&self) -> Option<&Arc<SymbolTable>> {
        self.0.output_symbols()
    }

    fn set_input_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.0.set_input_symbols(symt)
    }

    fn set_output_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.0.set_output_symbols(symt)
    }

    fn take_input_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.0.take_input_symbols()
    }

    fn take_output_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.0.take_output_symbols()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::algorithms::compose::matchers::SortedMatcher;
    use crate::fst_impls::VectorFst;
    use crate::semirings::TropicalWeight;

    #[test]
    fn test_compose_fst_sync() {
        fn is_sync<T: Sync>() {}
        is_sync::<
            ComposeFst<
                TropicalWeight,
                SequenceComposeFilterBuilder<
                    _,
                    SortedMatcher<_, VectorFst<_>>,
                    SortedMatcher<_, VectorFst<_>>,
                >,
            >,
        >();
    }
}
