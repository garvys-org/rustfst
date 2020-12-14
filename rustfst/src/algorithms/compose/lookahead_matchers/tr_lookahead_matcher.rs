use std::borrow::Borrow;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;
use unsafe_unwrap::UnsafeUnwrap;

use crate::algorithms::compose::lookahead_matchers::{
    LookAheadMatcherData, LookaheadMatcher, MatcherFlagsTrait,
};
use crate::algorithms::compose::matchers::{IterItemMatcher, MatchType, Matcher, MatcherFlags};
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{Label, StateId, Tr, Trs, EPS_LABEL, NO_LABEL};

#[derive(Debug, Clone)]
pub struct TrLookAheadMatcher<W, F, B, M, MFT>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F>,
    M: Matcher<W, F, B>,
{
    // matcher fst
    fst: B,
    matcher: M,
    ghost: PhantomData<(W, F, MFT)>,
}

impl<W, F, B, M, MFT> Matcher<W, F, B> for TrLookAheadMatcher<W, F, B, M, MFT>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F> + Debug + Clone,
    M: Matcher<W, F, B>,
    MFT: MatcherFlagsTrait,
{
    type Iter = M::Iter;

    fn new(fst: B, match_type: MatchType) -> Result<Self> {
        Ok(Self {
            matcher: M::new(fst.clone(), match_type)?,
            fst,
            ghost: PhantomData,
        })
    }

    fn iter(&self, state: StateId, label: Label) -> Result<Self::Iter> {
        self.matcher.iter(state, label)
    }

    fn final_weight(&self, state: StateId) -> Result<Option<W>> {
        self.matcher.final_weight(state)
    }

    fn match_type(&self, test: bool) -> Result<MatchType> {
        self.matcher.match_type(test)
    }

    fn flags(&self) -> MatcherFlags {
        self.matcher.flags()
            | MatcherFlags::INPUT_LOOKAHEAD_MATCHER
            | MatcherFlags::OUTPUT_LOOKAHEAD_MATCHER
            | MFT::flags()
    }

    fn priority(&self, state: StateId) -> Result<usize> {
        self.matcher.priority(state)
    }

    fn fst(&self) -> &B {
        &self.fst
    }
}

impl<W, F, B, M, MFT> LookaheadMatcher<W, F, B> for TrLookAheadMatcher<W, F, B, M, MFT>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F> + Debug + Clone,
    M: Matcher<W, F, B>,
    MFT: MatcherFlagsTrait,
{
    // NullAddon
    type MatcherData = ();

    fn data(&self) -> Option<&Arc<Self::MatcherData>> {
        None
    }

    fn new_with_data(
        fst: B,
        match_type: MatchType,
        _data: Option<Arc<Self::MatcherData>>,
    ) -> Result<Self> {
        Self::new(fst, match_type)
    }

    fn create_data<F2: Fst<W>, BF2: Borrow<F2>>(
        _fst: BF2,
        _match_type: MatchType,
    ) -> Result<Option<Self::MatcherData>> {
        Ok(None)
    }

    fn init_lookahead_fst<LF: Fst<W>, BLF: Borrow<LF> + Clone>(
        &mut self,
        _lfst: &BLF,
    ) -> Result<()> {
        Ok(())
    }

    fn lookahead_fst<LF: Fst<W>, BLF: Borrow<LF>>(
        &self,
        matcher_state: StateId,
        lfst: &BLF,
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
        if self.fst.borrow().is_final(matcher_state)? && lfst.borrow().is_final(lfst_state)? {
            if !MFT::flags()
                .contains(MatcherFlags::LOOKAHEAD_WEIGHT | MatcherFlags::LOOKAHEAD_PREFIX)
            {
                return Ok(Some(la_matcher_data));
            }
            nprefix += 1;
            if MFT::flags().contains(MatcherFlags::LOOKAHEAD_WEIGHT) {
                unsafe {
                    let fw_matcher_state = self
                        .fst
                        .borrow()
                        .final_weight_unchecked(matcher_state)
                        .unsafe_unwrap();
                    let fw_lfst_state = lfst
                        .borrow()
                        .final_weight_unchecked(lfst_state)
                        .unsafe_unwrap();
                    la_matcher_data
                        .lookahead_weight
                        .plus_assign(fw_matcher_state.times(fw_lfst_state)?)?;
                }
            }
            result = true;
        }
        {
            let mut iter = self.iter(matcher_state, NO_LABEL)?.peekable();
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

        let match_type = self.match_type(false)?;
        for tr in lfst.borrow().get_trs(lfst_state)?.trs() {
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
                let mut iter = self.iter(matcher_state, label)?.peekable();
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

    fn lookahead_label(&self, state: StateId, label: Label) -> Result<bool> {
        let mut it = self.matcher.iter(state, label)?;
        Ok(it.next().is_some())
    }

    fn lookahead_prefix(&self, tr: &mut Tr<W>, la_matcher_data: &LookAheadMatcherData<W>) -> bool {
        la_matcher_data.default_lookahead_prefix(tr)
    }
}
