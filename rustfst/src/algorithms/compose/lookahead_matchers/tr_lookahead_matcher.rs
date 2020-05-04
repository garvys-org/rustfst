use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use anyhow::Result;
use unsafe_unwrap::UnsafeUnwrap;

use crate::algorithms::compose::lookahead_matchers::{LookaheadMatcher, MatcherFlagsTrait};
use crate::algorithms::compose::matchers::{IterItemMatcher, MatchType, Matcher, MatcherFlags};
use crate::fst_traits::{CoreFst, ExpandedFst, Fst};
use crate::semirings::Semiring;
use crate::{Tr, Label, StateId, EPS_LABEL, NO_LABEL, NO_STATE_ID};

#[derive(Debug)]
pub struct TrLookAheadMatcher<W: Semiring, M: Matcher<W>, MFT> {
    // matcher fst
    fst: Rc<M::F>,
    matcher: M,
    lookahead_weight: W,
    prefix_tr: Tr<W>,

    // Flags to customize the behaviour
    mft: PhantomData<MFT>,
}

impl<W: Semiring, M: Matcher<W>, MFT: MatcherFlagsTrait> Matcher<W>
    for TrLookAheadMatcher<W, M, MFT>
{
    type F = M::F;
    type Iter = M::Iter;

    fn new(fst: Rc<Self::F>, match_type: MatchType) -> Result<Self> {
        Ok(Self {
            fst: Rc::clone(&fst),
            matcher: M::new(fst, match_type)?,
            prefix_tr: Tr::new(0, 0, W::one(), NO_STATE_ID),
            lookahead_weight: W::one(),
            mft: PhantomData,
        })
    }

    fn iter(&self, state: usize, label: usize) -> Result<Self::Iter> {
        self.matcher.iter(state, label)
    }

    fn final_weight(&self, state: usize) -> Result<Option<*const W>> {
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

    fn priority(&self, state: usize) -> Result<usize> {
        self.matcher.priority(state)
    }

    fn fst(&self) -> Rc<Self::F> {
        Rc::clone(&self.fst)
    }
}

impl<W: Semiring, M: Matcher<W>, MFT: MatcherFlagsTrait> LookaheadMatcher<W>
    for TrLookAheadMatcher<W, M, MFT>
{
    // NullAddon
    type MatcherData = ();

    fn data(&self) -> Option<&Rc<RefCell<Self::MatcherData>>> {
        None
    }

    fn new_with_data(
        fst: Rc<Self::F>,
        match_type: MatchType,
        _data: Option<Rc<RefCell<Self::MatcherData>>>,
    ) -> Result<Self> {
        Self::new(fst, match_type)
    }

    fn create_data<F: ExpandedFst<W = W>>(
        _fst: &F,
        _match_type: MatchType,
    ) -> Result<Option<Rc<RefCell<Self::MatcherData>>>> {
        Ok(None)
    }

    fn init_lookahead_fst<LF: ExpandedFst<W = W>>(&mut self, _lfst: &Rc<LF>) -> Result<()> {
        Ok(())
    }

    fn lookahead_fst<LF: Fst<W = W>>(
        &mut self,
        matcher_state: StateId,
        lfst: &Rc<LF>,
        lfst_state: StateId,
    ) -> Result<bool> {
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
                            IterItemMatcher::Tr(a) => {
                                let a = unsafe { &*a };
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
                                IterItemMatcher::Tr(a) => {
                                    let a = unsafe { &*a };
                                    self.lookahead_weight
                                        .plus_assign(arc.weight.times(&a.weight)?)?
                                }
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

    fn lookahead_label(&self, state: StateId, label: Label) -> Result<bool> {
        let mut it = self.matcher.iter(state, label)?;
        Ok(it.next().is_some())
    }

    fn lookahead_prefix(&self, arc: &mut Tr<W>) -> bool {
        self.default_lookahead_prefix(arc)
    }

    fn lookahead_weight(&self) -> &W {
        &self.lookahead_weight
    }

    fn prefix_tr(&self) -> &Tr<W> {
        &self.prefix_tr
    }

    fn prefix_tr_mut(&mut self) -> &mut Tr<W> {
        &mut self.prefix_tr
    }

    fn lookahead_weight_mut(&mut self) -> &mut W {
        &mut self.lookahead_weight
    }
}
