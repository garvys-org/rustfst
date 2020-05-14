use anyhow::Result;

use crate::algorithms::compose::compose_filters::{
    ComposeFilter, ComposeFilterBuilder, SharedDataComposeFilter,
};
use crate::algorithms::compose::filter_states::{FilterState, IntegerFilterState};
use crate::algorithms::compose::matchers::{MatchType, Matcher};
use crate::fst_traits::{CoreFst, Fst};
use crate::semirings::Semiring;
use crate::{StateId, Tr, EPS_LABEL, NO_LABEL, NO_STATE_ID};
use std::cell::RefCell;
use std::sync::Arc;

#[derive(Debug)]
pub struct MatchComposeFilter<W: Semiring, M1: Matcher<W>, M2: Matcher<W>> {
    shared_data: Arc<SharedDataComposeFilter<W, M1, M2>>,
    /// Current fst1 state
    s1: StateId,
    /// Current fst2 state
    s2: StateId,
    /// Current filter state
    fs: IntegerFilterState,
    /// Only epsilons (and non-final) leaving s1 ?
    alleps1: bool,
    /// Only epsilons (and non-final) leaving s2 ?
    alleps2: bool,
    /// No epsilons leaving s1 ?
    noeps1: bool,
    /// No epsilons leaving s2 ?
    noeps2: bool,
}

#[derive(Debug)]
pub struct MatchComposeFilterBuilder<W: Semiring, M1: Matcher<W>, M2: Matcher<W>> {
    shared_data: Arc<SharedDataComposeFilter<W, M1, M2>>,
}

impl<W: Semiring, M1: Matcher<W>, M2: Matcher<W>> ComposeFilterBuilder<W>
    for MatchComposeFilterBuilder<W, M1, M2>
{
    type CF = MatchComposeFilter<W, M1, M2>;

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
        Ok(MatchComposeFilter::<W, M1, M2> {
            shared_data: Arc::clone(&self.shared_data),
            s1: NO_STATE_ID,
            s2: NO_STATE_ID,
            fs: <Self::CF as ComposeFilter<W>>::FS::new(NO_STATE_ID),
            alleps1: false,
            alleps2: false,
            noeps1: false,
            noeps2: false,
        })
    }
}

impl<W: Semiring, M1: Matcher<W>, M2: Matcher<W>> ComposeFilter<W>
    for MatchComposeFilter<W, M1, M2>
{
    type M1 = M1;
    type M2 = M2;
    type FS = IntegerFilterState;

    fn start(&self) -> Self::FS {
        Self::FS::new(0)
    }

    fn set_state(&mut self, s1: usize, s2: usize, filter_state: &Self::FS) -> Result<()> {
        if !(self.s1 == s1 && self.s2 == s2 && &self.fs == filter_state) {
            self.s1 = s1;
            self.s2 = s2;
            self.fs = filter_state.clone();

            let fst1 = self.shared_data.matcher1.fst();
            let fst2 = self.shared_data.matcher2.fst();

            let na1 = fst1.num_trs(s1)?;
            let na2 = fst2.num_trs(s2)?;

            let ne1 = fst1.num_output_epsilons(s1)?;
            let ne2 = fst2.num_input_epsilons(s2)?;

            let f1 = fst1.is_final(s1)?;
            let f2 = fst2.is_final(s2)?;

            self.alleps1 = na1 == ne1 && !f1;
            self.alleps2 = na2 == ne2 && !f2;

            self.noeps1 = ne1 == 0;
            self.noeps2 = ne2 == 0;
        }
        Ok(())
    }

    fn filter_tr(&mut self, arc1: &mut Tr<W>, arc2: &mut Tr<W>) -> Result<Self::FS> {
        let res = if arc2.ilabel == NO_LABEL {
            // EPSILON in FST1
            if self.fs == Self::FS::new(0) {
                if self.noeps2 {
                    Self::FS::new(0)
                } else if self.alleps2 {
                    Self::FS::new_no_state()
                } else {
                    Self::FS::new(1)
                }
            } else {
                if self.fs == Self::FS::new(1) {
                    Self::FS::new(1)
                } else {
                    Self::FS::new_no_state()
                }
            }
        } else if arc1.olabel == NO_LABEL {
            // Epsilon in FST2
            if self.fs == Self::FS::new(0) {
                if self.noeps1 {
                    Self::FS::new(0)
                } else if self.alleps1 {
                    Self::FS::new_no_state()
                } else {
                    Self::FS::new(2)
                }
            } else {
                if self.fs == Self::FS::new(2) {
                    Self::FS::new(2)
                } else {
                    Self::FS::new_no_state()
                }
            }
        } else if arc1.olabel == EPS_LABEL {
            // Epsilon in both
            if self.fs == Self::FS::new(0) {
                Self::FS::new(0)
            } else {
                Self::FS::new_no_state()
            }
        } else {
            // Both are non-epsilons
            Self::FS::new(0)
        };
        Ok(res)
    }

    fn filter_final(&self, _w1: &mut W, _w2: &mut W) -> Result<()> {
        Ok(())
    }

    fn get_shared_data(&self) -> &Arc<SharedDataComposeFilter<W, Self::M1, Self::M2>> {
        &self.shared_data
    }
}
