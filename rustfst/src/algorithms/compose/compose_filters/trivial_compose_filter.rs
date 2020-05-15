use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::compose::compose_filters::{
    ComposeFilter, ComposeFilterBuilder, SharedDataComposeFilter,
};
use crate::algorithms::compose::filter_states::{FilterState, TrivialFilterState};
use crate::algorithms::compose::matchers::{MatchType, Matcher};
use crate::semirings::Semiring;
use crate::Tr;

#[derive(Debug)]
pub struct TrivialComposeFilter<W: Semiring, M1: Matcher<W>, M2: Matcher<W>> {
    shared_data: Arc<SharedDataComposeFilter<W, M1, M2>>,
}

#[derive(Debug)]
pub struct TrivialComposeFilterBuilder<W: Semiring, M1: Matcher<W>, M2: Matcher<W>> {
    shared_data: Arc<SharedDataComposeFilter<W, M1, M2>>,
}

impl<W: Semiring, M1: Matcher<W>, M2: Matcher<W>> ComposeFilterBuilder<W>
    for TrivialComposeFilterBuilder<W, M1, M2>
{
    type CF = TrivialComposeFilter<W, M1, M2>;

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
        let shared_data = SharedDataComposeFilter::new(matcher1, matcher2);
        Ok(Self {
            shared_data: Arc::new(shared_data),
        })
    }

    fn build(&self) -> Result<Self::CF> {
        Ok(TrivialComposeFilter::<W, M1, M2> {
            shared_data: Arc::clone(&self.shared_data),
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

    fn get_shared_data(&self) -> &Arc<SharedDataComposeFilter<W, Self::M1, Self::M2>> {
        &self.shared_data
    }
}
