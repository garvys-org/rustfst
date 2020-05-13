use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::compose::lookahead_matchers::{LookaheadMatcher, MatcherFlagsTrait};
use crate::algorithms::compose::matchers::{MatchType, Matcher, MatcherFlags};
use crate::algorithms::compose::{LabelReachable, LabelReachableData};
use crate::fst_traits::ExpandedFst;
use crate::semirings::Semiring;
use crate::{Tr, Trs, EPS_LABEL, NO_STATE_ID};

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLookAheadMatcher<W: Semiring, M: Matcher<W>, MFT> {
    // matcher fst
    fst: Arc<M::F>,
    matcher: M,
    lookahead_weight: W,
    prefix_tr: Tr<W>,
    mft: PhantomData<MFT>,
    reachable: Option<LabelReachable>,
    lfst_ptr: *const u32,
}

impl<W: Semiring + 'static, M: Matcher<W>, MFT: MatcherFlagsTrait> Matcher<W>
    for LabelLookAheadMatcher<W, M, MFT>
{
    type F = M::F;
    type Iter = M::Iter;

    fn new(fst: Arc<Self::F>, match_type: MatchType) -> Result<Self> {
        Self::new_with_data(fst, match_type, None)
    }

    fn iter(&self, state: usize, label: usize) -> Result<Self::Iter> {
        self.matcher.iter(state, label)
    }

    fn final_weight(&self, state: usize) -> Result<Option<W>> {
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

    fn priority(&self, state: usize) -> Result<usize> {
        self.matcher.priority(state)
    }

    fn fst(&self) -> Arc<Self::F> {
        Arc::clone(&self.fst)
    }
}

impl<W: Semiring + 'static, M: Matcher<W>, MFT: MatcherFlagsTrait> LookaheadMatcher<W>
    for LabelLookAheadMatcher<W, M, MFT>
{
    type MatcherData = LabelReachableData;

    fn data(&self) -> Option<&Arc<RefCell<Self::MatcherData>>> {
        if let Some(reachable) = &self.reachable {
            Some(reachable.data())
        } else {
            None
        }
    }

    fn new_with_data(
        fst: Arc<Self::F>,
        match_type: MatchType,
        data: Option<Arc<RefCell<Self::MatcherData>>>,
    ) -> Result<Self> {
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
        } else if let Some(d) = Self::create_data(&fst, match_type)? {
            reachable = Some(LabelReachable::new_from_data(d));
        }

        Ok(Self {
            fst: Arc::clone(&fst),
            matcher: M::new(fst, match_type)?,
            prefix_tr: Tr::new(0, 0, W::one(), NO_STATE_ID),
            lookahead_weight: W::one(),
            reachable,
            lfst_ptr: std::ptr::null(),
            mft: PhantomData,
        })
    }

    fn create_data<F: ExpandedFst<W>>(
        fst: &F,
        match_type: MatchType,
    ) -> Result<Option<Arc<RefCell<Self::MatcherData>>>> {
        let reach_input = match_type == MatchType::MatchInput;
        if (reach_input && MFT::flags().contains(MatcherFlags::INPUT_LOOKAHEAD_MATCHER))
            || (!reach_input && MFT::flags().contains(MatcherFlags::OUTPUT_LOOKAHEAD_MATCHER))
        {
            Ok(Some(LabelReachable::new(fst, reach_input)?.shared_data()))
        } else {
            Ok(None)
        }
    }

    fn init_lookahead_fst<LF: ExpandedFst<W>>(&mut self, lfst: &Arc<LF>) -> Result<()> {
        let lfst_ptr = Arc::into_raw(Arc::clone(lfst)) as *const LF as *const u32;
        self.lfst_ptr = lfst_ptr;
        let reach_input = self.match_type() == MatchType::MatchOutput;
        if let Some(reachable) = &mut self.reachable {
            reachable.reach_init(lfst, reach_input)?;
        }
        Ok(())
    }

    fn lookahead_fst<LF: ExpandedFst<W>>(
        &mut self,
        matcher_state: usize,
        lfst: &Arc<LF>,
        lfst_state: usize,
    ) -> Result<bool> {
        // InitLookAheadFst
        let lfst_ptr = Arc::into_raw(Arc::clone(&lfst)) as *const LF as *const u32;
        if lfst_ptr != self.lfst_ptr {
            self.init_lookahead_fst(lfst)?;
        }

        // LookAheadFst
        self.clear_lookahead_weight();
        self.clear_lookahead_prefix();
        if let Some(reachable) = &self.reachable {
            let mut compute_weight = MFT::flags().contains(MatcherFlags::LOOKAHEAD_WEIGHT);
            let compute_prefix = MFT::flags().contains(MatcherFlags::LOOKAHEAD_PREFIX);
            let reach_tr = reachable.reach(
                matcher_state,
                lfst.get_trs(lfst_state)?,
                0,
                lfst.num_trs(lfst_state)?,
                compute_weight,
            )?;
            let reach_tr_bool = reach_tr.is_some();
            let lfinal = lfst.final_weight(lfst_state)?;
            let reach_final = lfinal.is_some()
                && !lfinal.clone().unwrap().is_zero()
                && reachable.reach_final(matcher_state)?;
            if let Some((reach_begin, reach_end, reach_weight)) = reach_tr {
                if compute_prefix && (reach_end - reach_begin) == 1 && !reach_final {
                    let trs_owner = lfst.get_trs(lfst_state)?;
                    let tr = trs_owner.trs().iter().skip(reach_begin).next().unwrap();
                    self.set_lookahead_prefix(tr.clone());
                    compute_weight = false;
                } else {
                    self.set_lookahead_weight(reach_weight);
                }
            }
            if reach_final && compute_weight {
                if reach_tr_bool {
                    self.lookahead_weight_mut().plus_assign(lfinal.unwrap())?;
                } else {
                    self.set_lookahead_weight(lfinal.unwrap().clone());
                }
            }
            Ok(reach_tr_bool || reach_final)
        } else {
            Ok(true)
        }
    }

    fn lookahead_label(&self, current_state: usize, label: usize) -> Result<bool> {
        if label == EPS_LABEL {
            return Ok(true);
        }
        if let Some(reachable) = &self.reachable {
            reachable.reach_label(current_state, label)
        } else {
            Ok(true)
        }
    }

    fn lookahead_prefix(&self, tr: &mut Tr<W>) -> bool {
        self.default_lookahead_prefix(tr)
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
