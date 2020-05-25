use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::compose::compose_filters::{ComposeFilter, ComposeFilterBuilder};
use crate::algorithms::compose::filter_states::FilterState;
use crate::algorithms::compose::lookahead_filters::lookahead_selector::{
    selector, MatchTypeTrait, Selector,
};
use crate::algorithms::compose::lookahead_filters::{
    lookahead_match_type, LookAheadComposeFilterTrait,
};
use crate::algorithms::compose::lookahead_matchers::{LookAheadMatcherData, LookaheadMatcher};
use crate::algorithms::compose::matchers::MatcherFlags;
use crate::algorithms::compose::matchers::{MatchType, Matcher};
use crate::fst_traits::ExpandedFst;
use crate::semirings::Semiring;
use crate::{Tr, EPS_LABEL};

#[derive(Clone, Debug)]
pub struct LookAheadComposeFilter<
    W: Semiring,
    CF: LookAheadComposeFilterTrait<W>,
    SMT: MatchTypeTrait,
> where
    CF::M1: LookaheadMatcher<W>,
    CF::M2: LookaheadMatcher<W>,
{
    filter: CF,
    lookahead_type: MatchType,
    flags: MatcherFlags,
    lookahead_tr: bool,
    smt: PhantomData<SMT>,
    w: PhantomData<W>,
    selector: Selector,
    la_matcher_data: Option<LookAheadMatcherData<W>>,
}

#[derive(Debug, Clone)]
pub struct LookAheadComposeFilterBuilder<W, CFB, SMT>
where
    W: Semiring,
    CFB: ComposeFilterBuilder<W>,
    CFB::CF: LookAheadComposeFilterTrait<W>,
    SMT: MatchTypeTrait,
    <CFB::CF as ComposeFilter<W>>::M1: LookaheadMatcher<W>,
    <CFB::CF as ComposeFilter<W>>::M2: LookaheadMatcher<W>,
{
    filter_builder: CFB,
    w: PhantomData<W>,
    smt: PhantomData<SMT>,
    lookahead_type: MatchType,
    flags: MatcherFlags,
    selector: Selector,
}

impl<W, F1, F2, M1, M2, CF, CFB, SMT> ComposeFilterBuilder<W>
    for LookAheadComposeFilterBuilder<W, CFB, SMT>
where
    W: Semiring,
    F1: ExpandedFst<W>,
    F2: ExpandedFst<W>,
    M1: Matcher<W, F = F1> + LookaheadMatcher<W>,
    M2: Matcher<W, F = F2> + LookaheadMatcher<W>,
    CF: ComposeFilter<W, M1 = M1, M2 = M2> + LookAheadComposeFilterTrait<W>,
    CFB: ComposeFilterBuilder<W, M1 = M1, M2 = M2, CF = CF>,
    SMT: MatchTypeTrait,
{
    type CF = LookAheadComposeFilter<W, CF, SMT>;
    type M1 = M1;
    type M2 = M2;

    fn new(
        fst1: Arc<<<Self::CF as ComposeFilter<W>>::M1 as Matcher<W>>::F>,
        fst2: Arc<<<Self::CF as ComposeFilter<W>>::M2 as Matcher<W>>::F>,
        matcher1: Option<Self::M1>,
        matcher2: Option<Self::M2>,
    ) -> Result<Self>
    where
        Self: Sized,
    {
        let mut matcher1 = matcher1
            .unwrap_or_else(|| Matcher::new(Arc::clone(&fst1), MatchType::MatchOutput).unwrap());
        let mut matcher2 = matcher2
            .unwrap_or_else(|| Matcher::new(Arc::clone(&fst2), MatchType::MatchInput).unwrap());

        let lookahead_type = if SMT::match_type() == MatchType::MatchBoth {
            lookahead_match_type(&matcher1, &matcher2)
        } else {
            SMT::match_type()
        };

        let flags = if lookahead_type == MatchType::MatchOutput {
            matcher1.flags()
        } else {
            matcher2.flags()
        };

        if lookahead_type == MatchType::MatchNone {
            bail!(
                "LookAheadComposeFilter: 1st argument cannot match/look-ahead on output \
                labels and 2nd argument cannot match/look-ahead on input labels"
            )
        }

        let selector = selector(SMT::match_type(), lookahead_type);

        match selector {
            Selector::Fst1Matcher2 => {
                matcher2.init_lookahead_fst(&fst1)?;
            }
            Selector::Fst2Matcher1 => {
                matcher1.init_lookahead_fst(&fst2)?;
            }
        };

        Ok(Self {
            filter_builder: CFB::new(fst1, fst2, Some(matcher1), Some(matcher2))?,
            w: PhantomData,
            smt: PhantomData,
            lookahead_type,
            flags,
            selector,
        })
    }

    fn build(&self) -> Result<Self::CF> {
        let filter = self.filter_builder.build()?;

        Ok(LookAheadComposeFilter::<W, CFB::CF, SMT> {
            lookahead_type: self.lookahead_type,
            flags: self.flags,
            smt: PhantomData,
            lookahead_tr: false,
            w: PhantomData,
            selector: self.selector,
            filter,
            la_matcher_data: None,
        })
    }
}

impl<W: Semiring, CF: LookAheadComposeFilterTrait<W>, SMT: MatchTypeTrait>
    LookAheadComposeFilter<W, CF, SMT>
where
    CF::M1: LookaheadMatcher<W>,
    CF::M2: LookaheadMatcher<W>,
{
    fn lookahead_filter_tr(
        &mut self,
        arca: &mut Tr<W>,
        arcb: &mut Tr<W>,
        fs: &CF::FS,
    ) -> Result<CF::FS> {
        let labela = if self.lookahead_output() {
            arca.olabel
        } else {
            arca.ilabel
        };
        if labela != EPS_LABEL && !self.flags.contains(MatcherFlags::LOOKAHEAD_NON_EPSILONS) {
            return Ok(fs.clone());
        }
        if labela == EPS_LABEL && !self.flags.contains(MatcherFlags::LOOKAHEAD_EPSILONS) {
            return Ok(fs.clone());
        }
        self.lookahead_tr = true;

        self.la_matcher_data = match self.selector() {
            Selector::Fst1Matcher2 => {
                let fst = self.fst1();
                let matcher = self.matcher2();
                matcher.lookahead_fst(arca.nextstate, fst, arcb.nextstate)?
            }
            Selector::Fst2Matcher1 => {
                let fst = self.fst2();
                let matcher = self.matcher1();
                matcher.lookahead_fst(arca.nextstate, fst, arcb.nextstate)?
            }
        };

        if self.la_matcher_data.is_some() {
            Ok(fs.clone())
        } else {
            Ok(CF::FS::new_no_state())
        }
    }
}

impl<W: Semiring, CF: LookAheadComposeFilterTrait<W>, SMT: MatchTypeTrait> ComposeFilter<W>
    for LookAheadComposeFilter<W, CF, SMT>
where
    CF::M1: LookaheadMatcher<W>,
    CF::M2: LookaheadMatcher<W>,
{
    type M1 = CF::M1;
    type M2 = CF::M2;
    type FS = CF::FS;

    fn start(&self) -> Self::FS {
        self.filter.start()
    }

    fn set_state(&mut self, s1: usize, s2: usize, filter_state: &Self::FS) -> Result<()> {
        self.filter.set_state(s1, s2, filter_state)
    }

    fn filter_tr(&mut self, arc1: &mut Tr<W>, arc2: &mut Tr<W>) -> Result<Self::FS> {
        self.lookahead_tr = false;
        let fs = self.filter.filter_tr(arc1, arc2)?;
        if fs == CF::FS::new_no_state() {
            return Ok(CF::FS::new_no_state());
        }
        if self.lookahead_output() {
            self.lookahead_filter_tr(arc1, arc2, &fs)
        } else {
            self.lookahead_filter_tr(arc2, arc1, &fs)
        }
    }

    fn filter_final(&self, w1: &mut W, w2: &mut W) -> Result<()> {
        self.filter.filter_final(w1, w2)
    }

    fn matcher1(&self) -> &Self::M1 {
        self.filter.matcher1()
    }

    fn matcher2(&self) -> &Self::M2 {
        self.filter.matcher2()
    }

    fn matcher1_shared(&self) -> &Arc<Self::M1> {
        self.filter.matcher1_shared()
    }

    fn matcher2_shared(&self) -> &Arc<Self::M2> {
        self.filter.matcher2_shared()
    }
}

impl<W: Semiring, CF: LookAheadComposeFilterTrait<W>, SMT: MatchTypeTrait>
    LookAheadComposeFilterTrait<W> for LookAheadComposeFilter<W, CF, SMT>
where
    CF::M1: LookaheadMatcher<W>,
    CF::M2: LookaheadMatcher<W>,
{
    fn lookahead_flags(&self) -> MatcherFlags {
        self.flags
    }

    fn lookahead_tr(&self) -> bool {
        self.lookahead_tr
    }

    fn lookahead_type(&self) -> MatchType {
        self.lookahead_type
    }

    fn lookahead_output(&self) -> bool {
        if SMT::match_type() == MatchType::MatchOutput {
            true
        } else if SMT::match_type() == MatchType::MatchInput {
            false
        } else if self.lookahead_type == MatchType::MatchOutput {
            true
        } else {
            false
        }
    }

    fn selector(&self) -> &Selector {
        &self.selector
    }

    fn lookahead_matcher_data(&self) -> Option<&LookAheadMatcherData<W>> {
        self.la_matcher_data.as_ref()
    }
}
