use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;
use unsafe_unwrap::UnsafeUnwrap;

use crate::algorithms::compose::lookahead_matchers::{
    LookAheadMatcherData, LookaheadMatcher, MatcherFlagsTrait,
};
use crate::algorithms::compose::matchers::{IterItemMatcher, MatchType, Matcher, MatcherFlags};
use crate::fst_traits::{CoreFst, Fst};
use crate::semirings::Semiring;
use crate::{Label, StateId, Tr, Trs, EPS_LABEL, NO_LABEL};

#[derive(Debug, Clone)]
pub struct TrLookAheadMatcher<W: Semiring, M: Matcher<W>, MFT> {
    // matcher fst
    matcher: M,
    // Flags to customize the behaviour
    mft: PhantomData<(W, MFT)>,
}

impl<W: Semiring, M: Matcher<W>, MFT: MatcherFlagsTrait> Matcher<W>
    for TrLookAheadMatcher<W, M, MFT>
{
    type Iter = M::Iter;

    fn new(fst: &impl Fst<W>, match_type: MatchType) -> Result<Self> {
        Ok(Self {
            matcher: M::new(fst, match_type)?,
            mft: PhantomData,
        })
    }

    fn iter(&self, fst: &impl Fst<W>, state: usize, label: usize) -> Result<Self::Iter> {
        self.matcher.iter(fst, state, label)
    }

    fn final_weight(&self, fst: &impl Fst<W>, state: usize) -> Result<Option<W>> {
        self.matcher.final_weight(fst, state)
    }

    fn match_type(&self, fst: &impl Fst<W>, test: bool) -> Result<MatchType> {
        self.matcher.match_type(fst, test)
    }

    fn flags(&self, fst: &impl Fst<W>) -> MatcherFlags {
        self.matcher.flags(fst)
            | MatcherFlags::INPUT_LOOKAHEAD_MATCHER
            | MatcherFlags::OUTPUT_LOOKAHEAD_MATCHER
            | MFT::flags()
    }

    fn priority(&self, state: usize) -> Result<usize> {
        self.matcher.priority(state)
    }
}

impl<W: Semiring, M: Matcher<W>, MFT: MatcherFlagsTrait> LookaheadMatcher<W>
    for TrLookAheadMatcher<W, M, MFT>
{
    // NullAddon
    type MatcherData = ();

    fn data(&self) -> Option<&Arc<Self::MatcherData>> {
        None
    }

    fn new_with_data(
        self_fst: &impl Fst<W>,
        fst: &impl Fst<W>,
        match_type: MatchType,
        _data: Option<Arc<Self::MatcherData>>,
    ) -> Result<Self> {
        Self::new(fst, match_type)
    }

    fn create_data(
        _fst2: &impl Fst<W>,
        _fst: &impl Fst<W>,
        _match_type: MatchType,
    ) -> Result<Option<Self::MatcherData>> {
        Ok(None)
    }

    fn init_lookahead_fst(&mut self, self_fst: &impl Fst<W>, lfst: &impl Fst<W>) -> Result<()> {
        Ok(())
    }

    fn lookahead_fst(
        &self,
        self_fst: &impl Fst<W>,
        matcher_state: StateId,
        lfst: &impl Fst<W>,
        lfst_state: StateId,
    ) -> Result<Option<LookAheadMatcherData<W>>> {
        let mut result = false;
        let mut nprefix = 0;
        let mut la_matcher_data = LookAheadMatcherData::default();
        if MFT::flags().contains(MatcherFlags::LOOKAHEAD_WEIGHT) {
            la_matcher_data.clear_lookahead_weight();
        }
        if MFT::flags().contains(MatcherFlags::LOOKAHEAD_PREFIX) {
            la_matcher_data.clear_lookahead_prefix();
        }
        if self_fst.is_final(matcher_state)? && lfst.is_final(lfst_state)? {
            if !MFT::flags()
                .contains(MatcherFlags::LOOKAHEAD_WEIGHT | MatcherFlags::LOOKAHEAD_PREFIX)
            {
                return Ok(Some(la_matcher_data));
            }
            nprefix += 1;
            if MFT::flags().contains(MatcherFlags::LOOKAHEAD_WEIGHT) {
                unsafe {
                    let fw_matcher_state =
                        self_fst.final_weight_unchecked(matcher_state).unsafe_unwrap();
                    let fw_lfst_state = lfst.final_weight_unchecked(lfst_state).unsafe_unwrap();
                    la_matcher_data
                        .lookahead_weight
                        .plus_assign(fw_matcher_state.times(fw_lfst_state)?)?;
                }
            }
            result = true;
        }
        {
            let mut iter = self.iter(self_fst, matcher_state, NO_LABEL)?.peekable();
            if iter.peek().is_some() {
                if !MFT::flags()
                    .contains(MatcherFlags::LOOKAHEAD_WEIGHT | MatcherFlags::LOOKAHEAD_PREFIX)
                {
                    return Ok(Some(la_matcher_data));
                }
                nprefix += 1;
                if MFT::flags().contains(MatcherFlags::LOOKAHEAD_WEIGHT) {
                    for tr in iter {
                        match tr {
                            IterItemMatcher::Tr(a) => {
                                la_matcher_data.lookahead_weight.plus_assign(&a.weight)?
                            }
                            IterItemMatcher::EpsLoop => {
                                la_matcher_data.lookahead_weight.plus_assign(W::one())?
                            }
                        };
                    }
                }
                result = true;
            }
        }

        let match_type = self.match_type(self_fst, false)?;
        for tr in lfst.get_trs(lfst_state)?.trs() {
            let label = match match_type {
                MatchType::MatchInput => tr.olabel,
                MatchType::MatchOutput => tr.ilabel,
                _ => bail!("Bad match type"),
            };
            if label == EPS_LABEL {
                if !MFT::flags()
                    .contains(MatcherFlags::LOOKAHEAD_WEIGHT | MatcherFlags::LOOKAHEAD_PREFIX)
                {
                    return Ok(Some(la_matcher_data));
                }
                if !MFT::flags().contains(MatcherFlags::LOOKAHEAD_NON_EPSILON_PREFIX) {
                    nprefix += 1;
                }
                if MFT::flags().contains(MatcherFlags::LOOKAHEAD_WEIGHT) {
                    la_matcher_data.lookahead_weight.plus_assign(&tr.weight)?;
                }
                result = true;
            } else {
                let mut iter = self.iter(self_fst, matcher_state, label)?.peekable();
                if iter.peek().is_some() {
                    if !MFT::flags()
                        .contains(MatcherFlags::LOOKAHEAD_WEIGHT | MatcherFlags::LOOKAHEAD_PREFIX)
                    {
                        return Ok(Some(la_matcher_data));
                    }
                    for matcher_value in iter {
                        nprefix += 1;
                        if MFT::flags().contains(MatcherFlags::LOOKAHEAD_WEIGHT) {
                            match matcher_value {
                                IterItemMatcher::Tr(a) => la_matcher_data
                                    .lookahead_weight
                                    .plus_assign(tr.weight.times(&a.weight)?)?,
                                IterItemMatcher::EpsLoop => la_matcher_data
                                    .lookahead_weight
                                    .plus_assign(tr.weight.times(W::one())?)?,
                            };
                        }
                        if MFT::flags().contains(MatcherFlags::LOOKAHEAD_PREFIX) && nprefix == 1 {
                            la_matcher_data.set_lookahead_prefix(tr.clone());
                        }
                    }
                    result = true;
                }
            }
        }

        if MFT::flags().contains(MatcherFlags::LOOKAHEAD_PREFIX) {
            if nprefix == 1 {
                la_matcher_data.clear_lookahead_weight();
            } else {
                la_matcher_data.clear_lookahead_prefix();
            }
        }

        if result {
            Ok(Some(la_matcher_data))
        } else {
            Ok(None)
        }
    }

    fn lookahead_label(&self, fst: &impl Fst<W>, state: StateId, label: Label) -> Result<bool> {
        let mut it = self.matcher.iter(fst, state, label)?;
        Ok(it.next().is_some())
    }

    fn lookahead_prefix(
        &self,
        fst: &impl Fst<W>,
        tr: &mut Tr<W>,
        la_matcher_data: &LookAheadMatcherData<W>,
    ) -> bool {
        la_matcher_data.default_lookahead_prefix(tr)
    }
}
