use std::borrow::Borrow;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::compose::lookahead_matchers::{
    LookAheadMatcherData, LookaheadMatcher, MatcherFlagsTrait,
};
use crate::algorithms::compose::matchers::{MatchType, Matcher, MatcherFlags};
use crate::algorithms::compose::{LabelReachable, LabelReachableData};
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{Label, StateId, Tr, Trs, EPS_LABEL};

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLookAheadMatcher<W, F, B, M, MFT>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F>,
    M: Matcher<W, F, B>,
{
    matcher: M,
    reachable: Option<LabelReachable>,
    ghost: PhantomData<(W, F, B, MFT)>,
}

impl<W, F, B, M, MFT> Matcher<W, F, B> for LabelLookAheadMatcher<W, F, B, M, MFT>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F> + Debug + Clone,
    M: Matcher<W, F, B>,
    MFT: MatcherFlagsTrait,
{
    type Iter = M::Iter;

    fn new(fst: B, match_type: MatchType) -> Result<Self> {
        Self::new_with_data(fst, match_type, None)
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

    fn priority(&self, state: StateId) -> Result<usize> {
        self.matcher.priority(state)
    }

    fn fst(&self) -> &B {
        self.matcher.fst()
    }
}

impl<W, F, B, M, MFT> LookaheadMatcher<W, F, B> for LabelLookAheadMatcher<W, F, B, M, MFT>
where
    W: Semiring + 'static,
    F: Fst<W>,
    B: Borrow<F> + Debug + Clone,
    M: Matcher<W, F, B>,
    MFT: MatcherFlagsTrait,
{
    type MatcherData = LabelReachableData;

    fn data(&self) -> Option<&Arc<Self::MatcherData>> {
        if let Some(reachable) = &self.reachable {
            Some(reachable.data())
        } else {
            None
        }
    }

    fn new_with_data(
        fst: B,
        match_type: MatchType,
        data: Option<Arc<Self::MatcherData>>,
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
            if reach_input == d.reach_input() {
                reachable = Some(LabelReachable::new_from_data(d));
            }
        } else if let Some(d) = Self::create_data(fst.clone(), match_type)? {
            reachable = Some(LabelReachable::new_from_data(Arc::new(d)));
        }

        Ok(Self {
            matcher: M::new(fst, match_type)?,
            reachable,
            ghost: PhantomData,
        })
    }

    fn create_data<F2: Fst<W>, BF2: Borrow<F2>>(
        fst: BF2,
        match_type: MatchType,
    ) -> Result<Option<Self::MatcherData>> {
        let reach_input = match_type == MatchType::MatchInput;
        if (reach_input && MFT::flags().contains(MatcherFlags::INPUT_LOOKAHEAD_MATCHER))
            || (!reach_input && MFT::flags().contains(MatcherFlags::OUTPUT_LOOKAHEAD_MATCHER))
        {
            Ok(Some(LabelReachable::compute_data(
                fst.borrow(),
                reach_input,
            )?))
        } else {
            Ok(None)
        }
    }

    fn init_lookahead_fst<LF: Fst<W>, BLF: Borrow<LF> + Clone>(
        &mut self,
        lfst: &BLF,
    ) -> Result<()> {
        let reach_input = self.match_type(false)? == MatchType::MatchOutput;
        if let Some(reachable) = &mut self.reachable {
            reachable.reach_init(lfst.borrow(), reach_input)?;
        }
        Ok(())
    }

    fn lookahead_fst<LF: Fst<W>, BLF: Borrow<LF>>(
        &self,
        matcher_state: StateId,
        lfst: &BLF,
        lfst_state: StateId,
    ) -> Result<Option<LookAheadMatcherData<W>>> {
        let lfst = lfst.borrow();
        // InitLookAheadFst
        // let lfst_ptr = Arc::into_raw(Arc::clone(&lfst)) as *const LF as *const u32;
        // if lfst_ptr != self.lfst_ptr {
        //     self.check_lookahead_fst(lfst)?;
        // }

        // LookAheadFst
        let mut la_matcher_data = LookAheadMatcherData::default();
        la_matcher_data.clear_lookahead_weight();
        la_matcher_data.clear_lookahead_prefix();
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
                    let tr = &trs_owner.trs()[reach_begin];
                    la_matcher_data.set_lookahead_prefix(tr.clone());
                    compute_weight = false;
                } else {
                    la_matcher_data.set_lookahead_weight(reach_weight);
                }
            }
            if reach_final && compute_weight {
                if reach_tr_bool {
                    la_matcher_data
                        .lookahead_weight
                        .plus_assign(lfinal.unwrap())?;
                } else {
                    la_matcher_data.set_lookahead_weight(lfinal.unwrap());
                }
            }
            if reach_tr_bool || reach_final {
                Ok(Some(la_matcher_data))
            } else {
                Ok(None)
            }
        } else {
            Ok(Some(la_matcher_data))
        }
    }

    fn lookahead_label(&self, current_state: StateId, label: Label) -> Result<bool> {
        if label == EPS_LABEL {
            return Ok(true);
        }
        if let Some(reachable) = &self.reachable {
            reachable.reach_label(current_state, label)
        } else {
            Ok(true)
        }
    }

    fn lookahead_prefix(&self, tr: &mut Tr<W>, la_matcher_data: &LookAheadMatcherData<W>) -> bool {
        la_matcher_data.default_lookahead_prefix(tr)
    }
}
