use std::hash::Hash;

use failure::Fallible;

use crate::algorithms::cache::{CacheImpl, FstImpl, StateTable};
use crate::algorithms::compose_filters::ComposeFilter;
use crate::algorithms::matchers::MatchType;
use crate::algorithms::matchers::Matcher;
use crate::fst_traits::{CoreFst, Fst};
use crate::semirings::Semiring;
use crate::StateId;

#[derive(Default, PartialEq, Eq, Clone, Hash, PartialOrd, Debug)]
struct ComposeStateTuple<FS> {
    fs: FS,
    s1: StateId,
    s2: StateId,
}

#[derive(Clone, PartialEq)]
struct ComposeFstImpl<
    'matcher,
    'fst,
    F1: Fst + 'fst,
    F2: Fst<W = F1::W> + 'fst,
    CF: ComposeFilter<'matcher, 'fst, F1, F2>,
> {
    fst1: &'fst F1,
    fst2: &'fst F2,
    compose_filter: CF,
    cache_impl: CacheImpl<F1::W>,
    state_table: StateTable<ComposeStateTuple<CF::FS>>,
    match_type: MatchType,
}

impl<
        'matcher,
        'fst: 'matcher,
        F1: Fst + 'fst,
        F2: Fst<W = F1::W> + 'fst,
        CF: ComposeFilter<'matcher, 'fst, F1, F2>,
    > FstImpl for ComposeFstImpl<'matcher, 'fst, F1, F2, CF>
where
    <F1 as CoreFst>::W: 'static,
{
    type W = F1::W;

    fn cache_impl_mut(&mut self) -> &mut CacheImpl<Self::W> {
        &mut self.cache_impl
    }

    fn cache_impl_ref(&self) -> &CacheImpl<Self::W> {
        &self.cache_impl
    }

    fn expand(&mut self, state: usize) -> Fallible<()> {
        unimplemented!()
    }

    fn compute_start(&mut self) -> Fallible<Option<StateId>> {
        let s1 = self.fst1.start();
        if s1.is_none() {
            return Ok(None);
        }
        let s1 = s1.unwrap();
        let s2 = self.fst2.start();
        if s2.is_none() {
            return Ok(None);
        }
        let s2 = s2.unwrap();
        let fs = self.compose_filter.start();
        let tuple = ComposeStateTuple { s1, s2, fs };
        Ok(Some(self.state_table.find_id(tuple)))
    }

    fn compute_final(&mut self, state: usize) -> Fallible<Option<Self::W>> {
        let tuple = self.state_table.find_tuple(state);

        let s1 = tuple.s1;
        let final1 = self.compose_filter.matcher1().final_weight(s1)?;
        if final1.is_none() {
            return Ok(None);
        }
        let mut final1 = final1.unwrap().clone();

        let s2 = tuple.s2;
        let final2 = self.compose_filter.matcher2().final_weight(s2)?;
        if final2.is_none() {
            return Ok(None);
        }
        let mut final2 = final2.unwrap().clone();

        self.compose_filter.set_state(s1, s2, &tuple.fs);
        self.compose_filter.filter_final(&mut final1, &mut final2);

        final1.times_assign(&final2)?;
        Ok(Some(final1))
    }
}
