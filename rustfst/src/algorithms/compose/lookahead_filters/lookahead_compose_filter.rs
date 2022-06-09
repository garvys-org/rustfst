use std::borrow::Borrow;
use std::fmt::Debug;
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
use crate::fst_properties::FstProperties;
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{StateId, Tr, EPS_LABEL};

#[derive(Clone, Debug)]
pub struct LookAheadComposeFilter<W, F1, F2, B1, B2, M1, M2, CF, SMT>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: LookaheadMatcher<W, F1, B1>,
    M2: LookaheadMatcher<W, F2, B2>,
    CF: LookAheadComposeFilterTrait<W, F1, F2, B1, B2, M1, M2>,
    SMT: MatchTypeTrait,
{
    filter: CF,
    lookahead_type: MatchType,
    flags: MatcherFlags,
    lookahead_tr: bool,
    selector: Selector,
    la_matcher_data: Option<LookAheadMatcherData<W>>,
    #[allow(clippy::type_complexity)]
    ghost: PhantomData<(W, F1, F2, B1, B2, M1, M2, SMT)>,
}

#[derive(Debug)]
pub struct LookAheadComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2, CFB, SMT>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: LookaheadMatcher<W, F1, B1>,
    M2: LookaheadMatcher<W, F2, B2>,
    CFB: ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>,
    CFB::CF: LookAheadComposeFilterTrait<W, F1, F2, B1, B2, M1, M2>,
    SMT: MatchTypeTrait,
{
    filter_builder: CFB,
    lookahead_type: MatchType,
    flags: MatcherFlags,
    selector: Selector,
    #[allow(clippy::type_complexity)]
    ghost: PhantomData<(W, F1, F2, B1, B2, M1, M2, SMT)>,
}

impl<W, F1, F2, B1, B2, M1, M2, CFB, SMT> Clone
    for LookAheadComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2, CFB, SMT>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: LookaheadMatcher<W, F1, B1>,
    M2: LookaheadMatcher<W, F2, B2>,
    CFB: ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>,
    CFB::CF: LookAheadComposeFilterTrait<W, F1, F2, B1, B2, M1, M2>,
    SMT: MatchTypeTrait,
{
    fn clone(&self) -> Self {
        LookAheadComposeFilterBuilder {
            filter_builder: self.filter_builder.clone(),
            lookahead_type: self.lookahead_type,
            flags: self.flags,
            selector: self.selector,
            ghost: PhantomData,
        }
    }
}

impl<W, F1, F2, B1, B2, M1, M2, CFB, SMT> ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>
    for LookAheadComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2, CFB, SMT>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug + Clone,
    B2: Borrow<F2> + Debug + Clone,
    M1: LookaheadMatcher<W, F1, B1>,
    M2: LookaheadMatcher<W, F2, B2>,
    CFB: ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>,
    CFB::CF: LookAheadComposeFilterTrait<W, F1, F2, B1, B2, M1, M2>,
    SMT: MatchTypeTrait,
    LookAheadComposeFilter<W, F1, F2, B1, B2, M1, M2, CFB::CF, SMT>:
        ComposeFilter<W, F1, F2, B1, B2, M1, M2>,
{
    type IM1 = M1;
    type IM2 = M2;
    type CF = LookAheadComposeFilter<W, F1, F2, B1, B2, M1, M2, CFB::CF, SMT>;

    fn new(fst1: B1, fst2: B2, matcher1: Option<M1>, matcher2: Option<M2>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut matcher1 =
            matcher1.unwrap_or_else(|| Matcher::new(fst1.clone(), MatchType::MatchOutput).unwrap());
        let mut matcher2 =
            matcher2.unwrap_or_else(|| Matcher::new(fst2.clone(), MatchType::MatchInput).unwrap());

        let lookahead_type = if SMT::match_type() == MatchType::MatchBoth {
            lookahead_match_type(&matcher1, &matcher2)?
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
            lookahead_type,
            flags,
            selector,
            ghost: PhantomData,
        })
    }

    fn build(&self) -> Result<Self::CF> {
        let filter = self.filter_builder.build()?;

        Ok(
            LookAheadComposeFilter::<W, F1, F2, B1, B2, M1, M2, CFB::CF, SMT> {
                lookahead_type: self.lookahead_type,
                flags: self.flags,
                lookahead_tr: false,
                selector: self.selector,
                filter,
                la_matcher_data: None,
                ghost: PhantomData,
            },
        )
    }
}

impl<W, F1, F2, B1, B2, M1, M2, CF, SMT> LookAheadComposeFilter<W, F1, F2, B1, B2, M1, M2, CF, SMT>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: LookaheadMatcher<W, F1, B1>,
    M2: LookaheadMatcher<W, F2, B2>,
    CF: LookAheadComposeFilterTrait<W, F1, F2, B1, B2, M1, M2>,
    SMT: MatchTypeTrait,
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
                let fst = self.matcher1().fst();
                let matcher = self.matcher2();
                matcher.lookahead_fst(arca.nextstate, fst, arcb.nextstate)?
            }
            Selector::Fst2Matcher1 => {
                let fst = self.matcher2().fst();
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

impl<W, F1, F2, B1, B2, M1, M2, CF, SMT> ComposeFilter<W, F1, F2, B1, B2, M1, M2>
    for LookAheadComposeFilter<W, F1, F2, B1, B2, M1, M2, CF, SMT>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: LookaheadMatcher<W, F1, B1>,
    M2: LookaheadMatcher<W, F2, B2>,
    CF: LookAheadComposeFilterTrait<W, F1, F2, B1, B2, M1, M2>,
    SMT: MatchTypeTrait,
{
    type FS = CF::FS;

    fn start(&self) -> Self::FS {
        self.filter.start()
    }

    fn set_state(&mut self, s1: StateId, s2: StateId, filter_state: &Self::FS) -> Result<()> {
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

    fn matcher1(&self) -> &M1 {
        self.filter.matcher1()
    }

    fn matcher2(&self) -> &M2 {
        self.filter.matcher2()
    }

    fn matcher1_shared(&self) -> &Arc<M1> {
        self.filter.matcher1_shared()
    }

    fn matcher2_shared(&self) -> &Arc<M2> {
        self.filter.matcher2_shared()
    }

    fn properties(&self, inprops: FstProperties) -> FstProperties {
        let outprops = self.filter.properties(inprops);
        if self.lookahead_type == MatchType::MatchNone {
            panic!("Error");
        }
        outprops
    }
}

impl<W, F1, F2, B1, B2, M1, M2, CF, SMT> LookAheadComposeFilterTrait<W, F1, F2, B1, B2, M1, M2>
    for LookAheadComposeFilter<W, F1, F2, B1, B2, M1, M2, CF, SMT>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: LookaheadMatcher<W, F1, B1>,
    M2: LookaheadMatcher<W, F2, B2>,
    CF: LookAheadComposeFilterTrait<W, F1, F2, B1, B2, M1, M2>,
    SMT: MatchTypeTrait,
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
        } else {
            self.lookahead_type == MatchType::MatchOutput
        }
    }

    fn selector(&self) -> &Selector {
        &self.selector
    }

    fn lookahead_matcher_data(&self) -> Option<&LookAheadMatcherData<W>> {
        self.la_matcher_data.as_ref()
    }
}
