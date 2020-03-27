use failure::Fallible;

use crate::algorithms::lookahead_matchers::{LookaheadMatcher, MatcherFlagsTrait};
use crate::algorithms::matchers::{IterItemMatcher, MatchType, Matcher, MatcherFlags};
use crate::fst_traits::{CoreFst, ExpandedFst, Fst};
use crate::semirings::Semiring;
use crate::{Arc, Label, StateId, EPS_LABEL, NO_LABEL, NO_STATE_ID};
use std::marker::PhantomData;
use unsafe_unwrap::UnsafeUnwrap;

#[derive(Debug)]
pub struct ArcLookAheadMatcher<'fst, W: Semiring, M: Matcher<'fst, W>, MFT> {
    // matcher fst
    fst: &'fst M::F,
    matcher: M,
    lookahead_weight: W,
    prefix_arc: Arc<W>,

    // Flags to customize the behaviour
    mft: PhantomData<MFT>,
}

impl<'fst, W: Semiring + 'fst, M: Matcher<'fst, W>, MFT: MatcherFlagsTrait> Matcher<'fst, W>
    for ArcLookAheadMatcher<'fst, W, M, MFT>
{
    type F = M::F;
    type Iter = M::Iter;

    fn new(fst: &'fst Self::F, match_type: MatchType) -> Fallible<Self> {
        Ok(Self {
            fst,
            matcher: M::new(fst, match_type)?,
            prefix_arc: Arc::new(0, 0, W::one(), NO_STATE_ID),
            lookahead_weight: W::one(),
            mft: PhantomData,
        })
    }

    fn iter(&self, state: usize, label: usize) -> Fallible<Self::Iter> {
        self.matcher.iter(state, label)
    }

    fn final_weight(&self, state: usize) -> Fallible<Option<&'fst W>> {
        self.matcher.final_weight(state)
    }

    fn match_type(&self) -> MatchType {
        self.matcher.match_type()
    }

    fn flags(&self) -> MatcherFlags {
        self.matcher.flags()
            | MatcherFlags::INPUT_LOOKAHEAD_MATCHER
            | MatcherFlags::OUTPUT_LOOKAHEAD_MATCHER
            | MFT::flags()
    }

    fn priority(&self, state: usize) -> Fallible<usize> {
        self.matcher.priority(state)
    }

    fn fst(&self) -> &'fst Self::F {
        &self.fst
    }
}

impl<'fst, W: Semiring + 'fst, M: Matcher<'fst, W>, MFT: MatcherFlagsTrait>
    LookaheadMatcher<'fst, W> for ArcLookAheadMatcher<'fst, W, M, MFT>
{
    // NullAddon
    type MatcherData = ();

    fn data(&self) -> Option<&Self::MatcherData> {
        None
    }

    fn new_with_data(
        fst: &'fst Self::F,
        match_type: MatchType,
        _data: Option<Self::MatcherData>,
    ) -> Fallible<Self> {
        Self::new(fst, match_type)
    }

    fn create_data(_fst: &Self::F, _match_type: MatchType) -> Option<Self::MatcherData> {
        None
    }

    fn init_lookahead_fst<LF: ExpandedFst<W = W>>(&mut self, _lfst: &LF) -> Fallible<()> {
        Ok(())
    }

    fn lookahead_fst<LF: Fst<W = W>>(
        &mut self,
        matcher_state: StateId,
        lfst: &LF,
        lfst_state: StateId,
    ) -> Fallible<bool> {
        let mut result = false;
        let mut nprefix = 0;
        if MFT::flags().contains(MatcherFlags::LOOKAHEAD_WEIGHT) {
            self.clear_lookahead_weight();
        }
        if MFT::flags().contains(MatcherFlags::LOOKAHEAD_PREFIX) {
            self.clear_lookahead_prefix();
        }
        if self.fst.is_final(matcher_state)? && lfst.is_final(lfst_state)? {
            if !MFT::flags()
                .contains(MatcherFlags::LOOKAHEAD_WEIGHT | MatcherFlags::LOOKAHEAD_PREFIX)
            {
                return Ok(true);
            }
            nprefix += 1;
            if MFT::flags().contains(MatcherFlags::LOOKAHEAD_WEIGHT) {
                unsafe {
                    let fw_matcher_state = self
                        .fst
                        .final_weight_unchecked(matcher_state)
                        .unsafe_unwrap();
                    let fw_lfst_state = lfst.final_weight_unchecked(lfst_state).unsafe_unwrap();
                    self.lookahead_weight
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
                    return Ok(true);
                }
                nprefix += 1;
                if MFT::flags().contains(MatcherFlags::LOOKAHEAD_WEIGHT) {
                    for arc in iter {
                        match arc {
                            IterItemMatcher::Arc(a) => {
                                self.lookahead_weight.plus_assign(&a.weight)?
                            }
                            IterItemMatcher::EpsLoop => {
                                self.lookahead_weight.plus_assign(W::one())?
                            }
                        };
                    }
                }
                result = true;
            }
        }

        let match_type = self.match_type();
        for arc in lfst.arcs_iter(lfst_state)? {
            let label = match match_type {
                MatchType::MatchInput => arc.olabel,
                MatchType::MatchOutput => arc.ilabel,
                _ => bail!("Bad match type"),
            };
            if label == EPS_LABEL {
                if !MFT::flags()
                    .contains(MatcherFlags::LOOKAHEAD_WEIGHT | MatcherFlags::LOOKAHEAD_PREFIX)
                {
                    return Ok(true);
                }
                if !MFT::flags().contains(MatcherFlags::LOOKAHEAD_NON_EPSILON_PREFIX) {
                    nprefix += 1;
                }
                if MFT::flags().contains(MatcherFlags::LOOKAHEAD_WEIGHT) {
                    self.lookahead_weight.plus_assign(&arc.weight)?;
                }
                result = true;
            } else {
                let mut iter = self.iter(matcher_state, label)?.peekable();
                if iter.peek().is_some() {
                    if !MFT::flags()
                        .contains(MatcherFlags::LOOKAHEAD_WEIGHT | MatcherFlags::LOOKAHEAD_PREFIX)
                    {
                        return Ok(true);
                    }
                    for matcher_value in iter {
                        nprefix += 1;
                        if MFT::flags().contains(MatcherFlags::LOOKAHEAD_WEIGHT) {
                            match matcher_value {
                                IterItemMatcher::Arc(a) => self
                                    .lookahead_weight
                                    .plus_assign(arc.weight.times(&a.weight)?)?,
                                IterItemMatcher::EpsLoop => self
                                    .lookahead_weight
                                    .plus_assign(arc.weight.times(W::one())?)?,
                            };
                        }
                        if MFT::flags().contains(MatcherFlags::LOOKAHEAD_PREFIX) && nprefix == 1 {
                            self.set_lookahead_prefix(arc.clone());
                        }
                    }
                    result = true;
                }
            }
        }

        if MFT::flags().contains(MatcherFlags::LOOKAHEAD_PREFIX) {
            if nprefix == 1 {
                self.clear_lookahead_weight();
            } else {
                self.clear_lookahead_prefix();
            }
        }

        Ok(result)
    }

    fn lookahead_label(&self, state: StateId, label: Label) -> Fallible<bool> {
        let mut it = self.matcher.iter(state, label)?;
        Ok(it.next().is_some())
    }

    fn lookahead_prefix(&self, arc: &mut Arc<W>) -> bool {
        self.default_lookahead_prefix(arc)
    }

    fn lookahead_weight(&self) -> &W {
        &self.lookahead_weight
    }

    fn prefix_arc(&self) -> &Arc<W> {
        &self.prefix_arc
    }

    fn prefix_arc_mut(&mut self) -> &mut Arc<W> {
        &mut self.prefix_arc
    }

    fn lookahead_weight_mut(&mut self) -> &mut W {
        &mut self.lookahead_weight
    }
}
