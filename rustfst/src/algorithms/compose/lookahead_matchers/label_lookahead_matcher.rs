use std::marker::PhantomData;

use failure::Fallible;

use crate::algorithms::compose::lookahead_matchers::{LookaheadMatcher, MatcherFlagsTrait};
use crate::algorithms::compose::matchers::{MatchType, Matcher, MatcherFlags};
use crate::algorithms::compose::{LabelReachable, LabelReachableData};
use crate::fst_traits::ExpandedFst;
use crate::semirings::Semiring;
use crate::{Arc, EPS_LABEL, NO_STATE_ID};
use failure::_core::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLookAheadMatcher<'fst, W: Semiring, M: Matcher<'fst, W>, MFT> {
    // matcher fst
    fst: &'fst M::F,
    matcher: M,
    lookahead_weight: W,
    prefix_arc: Arc<W>,
    mft: PhantomData<MFT>,
    reachable: Option<LabelReachable>,
    lfst_ptr: *const u32,
}

impl<'fst, W: Semiring + 'static, M: Matcher<'fst, W>, MFT: MatcherFlagsTrait> Matcher<'fst, W>
    for LabelLookAheadMatcher<'fst, W, M, MFT>
{
    type F = M::F;
    type Iter = M::Iter;

    fn new(fst: &'fst Self::F, match_type: MatchType) -> Fallible<Self> {
        Self::new_with_data(fst, match_type, None)
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
        if let Some(reachable) = &self.reachable {
            if reachable.reach_input() {
                self.matcher.flags() | MFT::flags() | MatcherFlags::INPUT_LOOKAHEAD_MATCHER
            } else {
                self.matcher.flags() | MFT::flags() | MatcherFlags::OUTPUT_LOOKAHEAD_MATCHER
            }
        } else {
            self.matcher.flags()
        }
    }

    fn priority(&self, state: usize) -> Fallible<usize> {
        self.matcher.priority(state)
    }

    fn fst(&self) -> &'fst Self::F {
        self.fst
    }
}

impl<'fst, W: Semiring + 'static, M: Matcher<'fst, W>, MFT: MatcherFlagsTrait>
    LookaheadMatcher<'fst, W> for LabelLookAheadMatcher<'fst, W, M, MFT>
{
    type MatcherData = LabelReachableData;

    fn data(&self) -> Option<&Rc<RefCell<Self::MatcherData>>> {
        if let Some(reachable) = &self.reachable {
            Some(reachable.data())
        } else {
            None
        }
    }

    fn new_with_data(
        fst: &'fst Self::F,
        match_type: MatchType,
        data: Option<Rc<RefCell<Self::MatcherData>>>,
    ) -> Fallible<Self> {
        if !(MFT::flags().contains(MatcherFlags::INPUT_LOOKAHEAD_MATCHER)
            | MFT::flags().contains(MatcherFlags::OUTPUT_LOOKAHEAD_MATCHER))
        {
            bail!(
                "LabelLookAheadMatcher : Bad Matcher flags : {:?}",
                MFT::flags()
            )
        }
        let reach_input = match_type == MatchType::MatchInput;

        let mut reachable = None;
        if let Some(d) = data {
            if reach_input == d.borrow().reach_input() {
                reachable = Some(LabelReachable::new_from_data(d.clone()));
            }
        } else if let Some(d) = Self::create_data(fst, match_type)? {
            reachable = Some(LabelReachable::new_from_data(d));
        }

        Ok(Self {
            fst,
            matcher: M::new(fst, match_type)?,
            prefix_arc: Arc::new(0, 0, W::one(), NO_STATE_ID),
            lookahead_weight: W::one(),
            reachable,
            lfst_ptr: std::ptr::null(),
            mft: PhantomData,
        })
    }

    fn create_data(
        fst: &Self::F,
        match_type: MatchType,
    ) -> Fallible<Option<Rc<RefCell<Self::MatcherData>>>> {
        let reach_input = match_type == MatchType::MatchInput;
        if (reach_input && MFT::flags().contains(MatcherFlags::INPUT_LOOKAHEAD_MATCHER))
            || (!reach_input && MFT::flags().contains(MatcherFlags::OUTPUT_LOOKAHEAD_MATCHER))
        {
            Ok(Some(LabelReachable::new(fst, reach_input)?.shared_data()))
        } else {
            Ok(None)
        }
    }

    fn init_lookahead_fst<LF: ExpandedFst<W = W>>(&mut self, lfst: &LF) -> Fallible<()> {
        let lfst_ptr = lfst as *const LF as *const u32;
        self.lfst_ptr = lfst_ptr;
        let reach_input = self.match_type() == MatchType::MatchOutput;
        if let Some(reachable) = &mut self.reachable {
            reachable.reach_init(lfst, reach_input)?;
        }
        Ok(())
    }

    fn lookahead_fst<LF: ExpandedFst<W = W>>(
        &mut self,
        matcher_state: usize,
        lfst: &LF,
        lfst_state: usize,
    ) -> Fallible<bool> {
        // InitLookAheadFst
        let lfst_ptr = lfst as *const LF as *const u32;
        if lfst_ptr != self.lfst_ptr {
            self.init_lookahead_fst(lfst)?;
        }

        // LookAheadFst
        self.clear_lookahead_weight();
        self.clear_lookahead_prefix();
        if let Some(reachable) = &self.reachable {
            let mut compute_weight = MFT::flags().contains(MatcherFlags::LOOKAHEAD_WEIGHT);
            let compute_prefix = MFT::flags().contains(MatcherFlags::LOOKAHEAD_PREFIX);
            let aiter = lfst.arcs_iter(lfst_state)?;
            let reach_arc = reachable.reach(
                matcher_state,
                aiter,
                0,
                lfst.num_arcs(lfst_state)?,
                compute_weight,
            )?;
            let reach_arc_bool = reach_arc.is_some();
            let lfinal = lfst.final_weight(lfst_state)?;
            let reach_final = lfinal.is_some()
                && !lfinal.unwrap().is_zero()
                && reachable.reach_final(matcher_state)?;
            if let Some((reach_begin, reach_end, reach_weight)) = reach_arc {
                std::dbg!(compute_prefix);
                std::dbg!(reach_final);
                if compute_prefix && (reach_end - reach_begin) == 1 && !reach_final {
                    let arc = lfst
                        .arcs_iter(lfst_state)?
                        .skip(reach_begin)
                        .next()
                        .unwrap();
                    self.set_lookahead_prefix(arc.clone());
                    compute_weight = false;
                } else {
                    self.set_lookahead_weight(reach_weight);
                }
            }
            if reach_final && compute_weight {
                if reach_arc_bool {
                    self.lookahead_weight_mut().plus_assign(lfinal.unwrap())?;
                } else {
                    self.set_lookahead_weight(lfinal.unwrap().clone());
                }
            }
            std::dbg!(reach_arc_bool);
            std::dbg!(reach_final);
            Ok(reach_arc_bool || reach_final)
        } else {
            Ok(true)
        }
    }

    fn lookahead_label(&self, current_state: usize, label: usize) -> Fallible<bool> {
        if label == EPS_LABEL {
            return Ok(true);
        }
        if let Some(reachable) = &self.reachable {
            reachable.reach_label(current_state, label)
        } else {
            Ok(true)
        }
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
