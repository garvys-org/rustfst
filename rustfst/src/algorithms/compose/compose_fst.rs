use anyhow::Result;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::path::Path;

use crate::algorithms::compose::compose_filters::{
    ComposeFilter, ComposeFilterBuilder, SequenceComposeFilterBuilder,
};
use crate::algorithms::compose::matchers::{GenericMatcher, Matcher};
use crate::algorithms::compose::{
    ComposeFstOp, ComposeFstOpOptions, ComposeFstOpState, ComposeStateTuple,
};
use crate::algorithms::lazy::{
    FstCache, LazyFst, SerializableCache, SerializableLazyFst, SimpleVecCache,
};
use crate::fst_properties::FstProperties;
use crate::fst_traits::{AllocableFst, CoreFst, Fst, FstIterator, MutableFst, StateIterator};
use crate::parsers::SerializeBinary;
use crate::semirings::{Semiring, SerializableSemiring};
use crate::{StateId, SymbolTable, TrsVec};
use std::sync::Arc;

type InnerLazyFst<W, F1, F2, B1, B2, M1, M2, CFB, Cache> =
    LazyFst<W, ComposeFstOp<W, F1, F2, B1, B2, M1, M2, CFB>, Cache>;

#[derive(Debug)]
pub struct ComposeFst<W, F1, F2, B1, B2, M1, M2, CFB, Cache = SimpleVecCache<W>>(
    InnerLazyFst<W, F1, F2, B1, B2, M1, M2, CFB, Cache>,
)
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug + Clone,
    B2: Borrow<F2> + Debug + Clone,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
    CFB: ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>;

impl<W, F1, F2, B1, B2, M1, M2, CFB, Cache> Clone
    for ComposeFst<W, F1, F2, B1, B2, M1, M2, CFB, Cache>
where
    W: Semiring + Clone,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug + Clone,
    B2: Borrow<F2> + Debug + Clone,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
    CFB: ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2> + Clone,
    Cache: FstCache<W> + Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

fn create_base<
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug + Clone,
    B2: Borrow<F2> + Debug + Clone,
>(
    fst1: B1,
    fst2: B2,
) -> Result<
    ComposeFstOp<
        W,
        F1,
        F2,
        B1,
        B2,
        GenericMatcher<W, F1, B1>,
        GenericMatcher<W, F2, B2>,
        SequenceComposeFilterBuilder<
            W,
            F1,
            F2,
            B1,
            B2,
            GenericMatcher<W, F1, B1>,
            GenericMatcher<W, F2, B2>,
        >,
    >,
> {
    // TODO: change this once Lookahead matchers are supported.
    let opts =
        ComposeFstOpOptions::<GenericMatcher<_, _, _>, GenericMatcher<_, _, _>, _, _>::default();
    let compose_impl = ComposeFstOp::new(fst1, fst2, opts)?;
    Ok(compose_impl)
}

impl<W, F1, F2, B1, B2, M1, M2, CFB, Cache> ComposeFst<W, F1, F2, B1, B2, M1, M2, CFB, Cache>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug + Clone,
    B2: Borrow<F2> + Debug + Clone,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
    CFB: ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>,
    Cache: FstCache<W>,
{
    pub fn new_with_options(
        fst1: B1,
        fst2: B2,
        opts: ComposeFstOpOptions<
            M1,
            M2,
            CFB,
            ComposeFstOpState<
                ComposeStateTuple<
                    <CFB::CF as ComposeFilter<W, F1, F2, B1, B2, CFB::IM1, CFB::IM2>>::FS,
                >,
            >,
        >,
    ) -> Result<Self>
    where
        Cache: Default,
    {
        let isymt = fst1.borrow().input_symbols().cloned();
        let osymt = fst2.borrow().output_symbols().cloned();
        let compose_impl = ComposeFstOp::new(fst1, fst2, opts)?;
        let fst_cache = Cache::default();
        let fst = LazyFst::from_op_and_cache(compose_impl, fst_cache, isymt, osymt);
        Ok(ComposeFst(fst))
    }

    pub fn new_with_options_and_cache(
        fst1: B1,
        fst2: B2,
        opts: ComposeFstOpOptions<
            M1,
            M2,
            CFB,
            ComposeFstOpState<
                ComposeStateTuple<
                    <CFB::CF as ComposeFilter<W, F1, F2, B1, B2, CFB::IM1, CFB::IM2>>::FS,
                >,
            >,
        >,
        fst_cache: Cache,
    ) -> Result<Self> {
        let isymt = fst1.borrow().input_symbols().cloned();
        let osymt = fst2.borrow().output_symbols().cloned();
        let compose_impl = ComposeFstOp::new(fst1, fst2, opts)?;
        let fst = LazyFst::from_op_and_cache(compose_impl, fst_cache, isymt, osymt);
        Ok(ComposeFst(fst))
    }

    // TODO: Change API, no really user friendly
    pub fn new(fst1: B1, fst2: B2) -> Result<Self>
    where
        Cache: Default,
    {
        Self::new_with_options(fst1, fst2, ComposeFstOpOptions::default())
    }

    /// Turns the Lazy FST into a static one.
    pub fn compute<F: MutableFst<W> + AllocableFst<W>>(&self) -> Result<F> {
        self.0.compute()
    }
}

impl<W, F1, F2, B1, B2>
    ComposeFst<
        W,
        F1,
        F2,
        B1,
        B2,
        GenericMatcher<W, F1, B1>,
        GenericMatcher<W, F2, B2>,
        SequenceComposeFilterBuilder<
            W,
            F1,
            F2,
            B1,
            B2,
            GenericMatcher<W, F1, B1>,
            GenericMatcher<W, F2, B2>,
        >,
    >
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug + Clone,
    B2: Borrow<F2> + Debug + Clone,
{
    pub fn new_auto(fst1: B1, fst2: B2) -> Result<Self> {
        let isymt = fst1.borrow().input_symbols().cloned();
        let osymt = fst2.borrow().output_symbols().cloned();
        let compose_impl = create_base(fst1, fst2)?;
        let fst_cache = SimpleVecCache::default();
        let fst = LazyFst::from_op_and_cache(compose_impl, fst_cache, isymt, osymt);
        Ok(ComposeFst(fst))
    }
}

impl<W, F1, F2, B1, B2, M1, M2, CFB, Cache> SerializableLazyFst
    for ComposeFst<W, F1, F2, B1, B2, M1, M2, CFB, Cache>
where
    W: SerializableSemiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug + Clone,
    B2: Borrow<F2> + Debug + Clone,
    Cache: FstCache<W> + SerializableCache,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
    CFB: ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>,
    <CFB::CF as ComposeFilter<W, F1, F2, B1, B2, CFB::IM1, CFB::IM2>>::FS: SerializeBinary,
{
    fn write<P: AsRef<Path>>(&self, cache_dir: P, op_state_dir: P) -> Result<()> {
        self.0.write(cache_dir, op_state_dir)
    }
}

impl<W, F1, F2, B1, B2, M1, M2, CFB, Cache> CoreFst<W>
    for ComposeFst<W, F1, F2, B1, B2, M1, M2, CFB, Cache>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug + Clone,
    B2: Borrow<F2> + Debug + Clone,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
    CFB: ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>,
    Cache: FstCache<W>,
{
    type TRS = TrsVec<W>;

    fn start(&self) -> Option<StateId> {
        self.0.start()
    }

    fn final_weight(&self, state_id: StateId) -> Result<Option<W>> {
        self.0.final_weight(state_id)
    }

    unsafe fn final_weight_unchecked(&self, state_id: StateId) -> Option<W> {
        self.0.final_weight_unchecked(state_id)
    }

    fn num_trs(&self, s: StateId) -> Result<usize> {
        self.0.num_trs(s)
    }

    unsafe fn num_trs_unchecked(&self, s: StateId) -> usize {
        self.0.num_trs_unchecked(s)
    }

    fn get_trs(&self, state_id: StateId) -> Result<Self::TRS> {
        self.0.get_trs(state_id)
    }

    unsafe fn get_trs_unchecked(&self, state_id: StateId) -> Self::TRS {
        self.0.get_trs_unchecked(state_id)
    }

    fn properties(&self) -> FstProperties {
        self.0.properties()
    }

    fn num_input_epsilons(&self, state: StateId) -> Result<usize> {
        self.0.num_input_epsilons(state)
    }

    fn num_output_epsilons(&self, state: StateId) -> Result<usize> {
        self.0.num_output_epsilons(state)
    }
}

impl<'a, W, F1, F2, B1, B2, M1, M2, CFB, Cache> StateIterator<'a>
    for ComposeFst<W, F1, F2, B1, B2, M1, M2, CFB, Cache>
where
    W: Semiring,
    F1: Fst<W> + 'a,
    F2: Fst<W> + 'a,
    B1: Borrow<F1> + Debug + Clone + 'a,
    B2: Borrow<F2> + Debug + Clone + 'a,
    M1: Matcher<W, F1, B1> + 'a,
    M2: Matcher<W, F2, B2> + 'a,
    CFB: ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2> + 'a,
    Cache: FstCache<W> + 'a,
{
    type Iter = <InnerLazyFst<W, F1, F2, B1, B2, M1, M2, CFB, Cache> as StateIterator<'a>>::Iter;

    fn states_iter(&'a self) -> Self::Iter {
        self.0.states_iter()
    }
}

impl<'a, W, F1, F2, B1, B2, M1, M2, CFB, Cache> FstIterator<'a, W>
    for ComposeFst<W, F1, F2, B1, B2, M1, M2, CFB, Cache>
where
    W: Semiring,
    F1: Fst<W> + 'a,
    F2: Fst<W> + 'a,
    B1: Borrow<F1> + Debug + Clone + 'a,
    B2: Borrow<F2> + Debug + Clone + 'a,
    M1: Matcher<W, F1, B1> + 'a,
    M2: Matcher<W, F2, B2> + 'a,
    CFB: ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2> + 'a,
    Cache: FstCache<W> + 'a,
{
    type FstIter =
        <InnerLazyFst<W, F1, F2, B1, B2, M1, M2, CFB, Cache> as FstIterator<'a, W>>::FstIter;

    fn fst_iter(&'a self) -> Self::FstIter {
        self.0.fst_iter()
    }
}

impl<W, F1, F2, B1, B2, M1, M2, CFB, Cache> Fst<W>
    for ComposeFst<W, F1, F2, B1, B2, M1, M2, CFB, Cache>
where
    W: Semiring,
    F1: Fst<W> + 'static,
    F2: Fst<W> + 'static,
    B1: Borrow<F1> + Debug + Clone + 'static,
    B2: Borrow<F2> + Debug + Clone + 'static,
    M1: Matcher<W, F1, B1> + 'static,
    M2: Matcher<W, F2, B2> + 'static,
    CFB: ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2> + 'static,
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
                VectorFst<_>,
                VectorFst<_>,
                Arc<_>,
                Arc<_>,
                SortedMatcher<_, _, _>,
                SortedMatcher<_, _, _>,
                SequenceComposeFilterBuilder<_, _, _, _, _, _, _>,
            >,
        >();
    }

    #[test]
    fn test_compose_fst_clonable() {
        fn is_clone<T: Clone>() {}
        is_clone::<
            ComposeFst<
                TropicalWeight,
                VectorFst<_>,
                VectorFst<_>,
                Arc<_>,
                Arc<_>,
                SortedMatcher<_, _, _>,
                SortedMatcher<_, _, _>,
                SequenceComposeFilterBuilder<_, _, _, _, _, _, _>,
            >,
        >();
    }
}
