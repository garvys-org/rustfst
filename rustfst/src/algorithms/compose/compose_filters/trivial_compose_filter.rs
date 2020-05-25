use std::sync::Arc;

use anyhow::Result;
use std::marker::PhantomData;

use crate::algorithms::compose::compose_filters::{ComposeFilter, ComposeFilterBuilder};
use crate::algorithms::compose::filter_states::{FilterState, TrivialFilterState};
use crate::algorithms::compose::matchers::{MatchType, Matcher};
use crate::semirings::Semiring;
use crate::Tr;

#[derive(Debug, Clone)]
pub struct TrivialComposeFilter<W: Semiring, M1: Matcher<W>, M2: Matcher<W>> {
    matcher1: Arc<M1>,
    matcher2: Arc<M2>,
    w: PhantomData<W>,
}

#[derive(Debug, Clone)]
pub struct TrivialComposeFilterBuilder<W: Semiring, M1: Matcher<W>, M2: Matcher<W>> {
    matcher1: Arc<M1>,
    matcher2: Arc<M2>,
    w: PhantomData<W>,
}

impl<W: Semiring, M1: Matcher<W>, M2: Matcher<W>> ComposeFilterBuilder<W>
    for TrivialComposeFilterBuilder<W, M1, M2>
{
    type CF = TrivialComposeFilter<W, M1, M2>;
    type M1 = M1;
    type M2 = M2;

    fn new(
        fst1: Arc<M1::F>,
        fst2: Arc<M2::F>,
        matcher1: Option<M1>,
        matcher2: Option<M2>,
    ) -> Result<Self> {
        let matcher1 =
            matcher1.unwrap_or_else(|| M1::new(Arc::clone(&fst1), MatchType::MatchOutput).unwrap());
        let matcher2 =
            matcher2.unwrap_or_else(|| M2::new(Arc::clone(&fst2), MatchType::MatchInput).unwrap());
        Ok(Self {
            matcher1: Arc::new(matcher1),
            matcher2: Arc::new(matcher2),
            w: PhantomData,
        })
    }

    fn build(&self) -> Result<Self::CF> {
        Ok(TrivialComposeFilter::<W, M1, M2> {
            matcher1: Arc::clone(&self.matcher1),
            matcher2: Arc::clone(&self.matcher2),
            w: PhantomData,
        })
    }
}

impl<W: Semiring, M1: Matcher<W>, M2: Matcher<W>> ComposeFilter<W>
    for TrivialComposeFilter<W, M1, M2>
{
    type M1 = M1;
    type M2 = M2;
    type FS = TrivialFilterState;

    fn start(&self) -> Self::FS {
        Self::FS::new(true)
    }

    fn set_state(&mut self, _s1: usize, _s2: usize, _filter_state: &Self::FS) -> Result<()> {
        Ok(())
    }

    fn filter_tr(&mut self, _tr1: &mut Tr<W>, _tr2: &mut Tr<W>) -> Result<Self::FS> {
        Ok(Self::FS::new(true))
    }

    fn filter_final(&self, _w1: &mut W, _w2: &mut W) -> Result<()> {
        Ok(())
    }

    fn matcher1(&self) -> &Self::M1 {
        &self.matcher1
    }

    fn matcher2(&self) -> &Self::M2 {
        &self.matcher2
    }

    fn matcher1_shared(&self) -> &Arc<Self::M1> {
        &self.matcher1
    }

    fn matcher2_shared(&self) -> &Arc<Self::M2> {
        &self.matcher2
    }
}
