use std::borrow::Borrow;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::compose::compose_filters::{ComposeFilter, ComposeFilterBuilder};
use crate::algorithms::compose::filter_states::{FilterState, IntegerFilterState, PairFilterState};
use crate::algorithms::compose::lookahead_filters::lookahead_selector::MatchTypeTrait;
use crate::algorithms::compose::lookahead_filters::lookahead_selector::Selector;
use crate::algorithms::compose::lookahead_filters::LookAheadComposeFilterTrait;
use crate::algorithms::compose::lookahead_matchers::LookaheadMatcher;
use crate::algorithms::compose::matchers::MatchType;
use crate::algorithms::compose::matchers::MatcherFlags;
use crate::algorithms::compose::matchers::{MultiEpsMatcher, MultiEpsMatcherFlags};
use crate::fst_properties::FstProperties;
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{Label, StateId, Tr, EPS_LABEL, NO_LABEL, NO_STATE_ID};

#[derive(Debug, Clone)]
pub struct PushLabelsComposeFilter<W, F1, F2, B1, B2, M1, M2, CF, SMT>
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
    matcher1: MultiEpsMatcher<W, F1, B1, M1>,
    matcher2: MultiEpsMatcher<W, F2, B2, M2>,
    filter: CF,
    fs: PairFilterState<CF::FS, IntegerFilterState>,
    ntrsa: usize,
    ghost: PhantomData<SMT>,
}

#[derive(Debug)]
pub struct PushLabelsComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2, CFB, SMT>
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
    #[allow(clippy::type_complexity)]
    ghost: PhantomData<(W, F1, F2, B1, B2, M1, M2, SMT)>,
}

impl<W, F1, F2, B1, B2, M1, M2, CFB, SMT> Clone
    for PushLabelsComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2, CFB, SMT>
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
        Self {
            filter_builder: self.filter_builder.clone(),
            ghost: PhantomData,
        }
    }
}

impl<W, F1, F2, B1, B2, M1, M2, CFB, SMT> ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>
    for PushLabelsComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2, CFB, SMT>
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
{
    type IM1 = MultiEpsMatcher<W, F1, B1, M1>;
    type IM2 = MultiEpsMatcher<W, F2, B2, M2>;

    type CF = PushLabelsComposeFilter<W, F1, F2, B1, B2, M1, M2, CFB::CF, SMT>;

    fn new(fst1: B1, fst2: B2, matcher1: Option<M1>, matcher2: Option<M2>) -> Result<Self>
    where
        Self: Sized,
    {
        let filter_builder = CFB::new(fst1, fst2, matcher1, matcher2)?;
        Ok(Self {
            filter_builder,
            ghost: PhantomData,
        })
    }

    fn build(&self) -> Result<Self::CF> {
        let filter = self.filter_builder.build()?;

        let matcher1 = MultiEpsMatcher::new_with_opts(
            filter.matcher1().fst().clone(),
            MatchType::MatchOutput,
            if filter.lookahead_output() {
                MultiEpsMatcherFlags::MULTI_EPS_LIST
            } else {
                MultiEpsMatcherFlags::MULTI_EPS_LOOP
            },
            Arc::clone(filter.matcher1_shared()),
        )?;
        let matcher2 = MultiEpsMatcher::new_with_opts(
            filter.matcher2().fst().clone(),
            MatchType::MatchInput,
            if filter.lookahead_output() {
                MultiEpsMatcherFlags::MULTI_EPS_LOOP
            } else {
                MultiEpsMatcherFlags::MULTI_EPS_LIST
            },
            Arc::clone(filter.matcher2_shared()),
        )?;
        Ok(Self::CF {
            fs: FilterState::new_no_state(),
            matcher1,
            matcher2,
            filter,
            ntrsa: 0,
            ghost: PhantomData,
        })
    }
}

impl<W, F1, F2, B1, B2, M1, M2, CF, SMT>
    ComposeFilter<W, F1, F2, B1, B2, MultiEpsMatcher<W, F1, B1, M1>, MultiEpsMatcher<W, F2, B2, M2>>
    for PushLabelsComposeFilter<W, F1, F2, B1, B2, M1, M2, CF, SMT>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug + Clone,
    B2: Borrow<F2> + Debug + Clone,
    M1: LookaheadMatcher<W, F1, B1>,
    M2: LookaheadMatcher<W, F2, B2>,
    CF: LookAheadComposeFilterTrait<W, F1, F2, B1, B2, M1, M2>,
    SMT: MatchTypeTrait,
{
    type FS = PairFilterState<CF::FS, IntegerFilterState>;

    fn start(&self) -> Self::FS {
        PairFilterState::new((self.filter.start(), FilterState::new(NO_LABEL)))
    }

    fn set_state(&mut self, s1: StateId, s2: StateId, filter_state: &Self::FS) -> Result<()> {
        self.fs = filter_state.clone();
        self.filter.set_state(s1, s2, filter_state.state1())?;
        if !self
            .filter
            .lookahead_flags()
            .contains(MatcherFlags::LOOKAHEAD_PREFIX)
        {
            return Ok(());
        }
        self.ntrsa = if self.lookahead_output() {
            self.filter.matcher1().fst().borrow().num_trs(s1)?
        } else {
            self.filter.matcher2().fst().borrow().num_trs(s2)?
        };
        let fs2 = filter_state.state2();
        let flabel = fs2.state();
        self.matcher1.clear_multi_eps_labels();
        self.matcher2.clear_multi_eps_labels();
        if *flabel != NO_LABEL {
            self.matcher1.add_multi_eps_label(*flabel)?;
            self.matcher2.add_multi_eps_label(*flabel)?;
        }
        Ok(())
    }

    fn filter_tr(&mut self, arc1: &mut Tr<W>, arc2: &mut Tr<W>) -> Result<Self::FS> {
        if !self
            .lookahead_flags()
            .contains(MatcherFlags::LOOKAHEAD_PREFIX)
        {
            return Ok(FilterState::new((
                self.filter.filter_tr(arc1, arc2)?,
                FilterState::new(NO_LABEL),
            )));
        }
        let fs2 = self.fs.state2();
        let flabel = fs2.state();
        if *flabel != NO_LABEL {
            if self.lookahead_output() {
                return self.pushed_label_filter_tr(arc1, arc2, *flabel);
            } else {
                return self.pushed_label_filter_tr(arc2, arc1, *flabel);
            }
        }
        let fs1 = self.filter.filter_tr(arc1, arc2)?;
        if fs1 == FilterState::new_no_state() {
            return Ok(FilterState::new_no_state());
        }
        if !self.lookahead_tr() {
            return Ok(FilterState::new((fs1, FilterState::new(NO_LABEL))));
        }
        if self.lookahead_output() {
            self.push_label_filter_tr(arc1, arc2, &fs1)
        } else {
            self.push_label_filter_tr(arc2, arc1, &fs1)
        }
    }

    fn filter_final(&self, w1: &mut W, w2: &mut W) -> Result<()> {
        self.filter.filter_final(w1, w2)?;
        if !self
            .lookahead_flags()
            .contains(MatcherFlags::LOOKAHEAD_PREFIX)
            || w1.is_zero()
        {
            return Ok(());
        }
        let fs2 = self.fs.state2();
        let flabel = fs2.state();
        if *flabel != NO_LABEL {
            *w1 = W::zero()
        }
        Ok(())
    }

    fn matcher1(&self) -> &MultiEpsMatcher<W, F1, B1, M1> {
        &self.matcher1
    }

    fn matcher2(&self) -> &MultiEpsMatcher<W, F2, B2, M2> {
        &self.matcher2
    }

    fn matcher1_shared(&self) -> &Arc<MultiEpsMatcher<W, F1, B1, M1>> {
        // Not supported at the moment as the MultiEpsMatcher is owned by the ComposeFilter
        unimplemented!()
    }

    fn matcher2_shared(&self) -> &Arc<MultiEpsMatcher<W, F2, B2, M2>> {
        // Not supported at the moment as the MultiEpsMatcher is owned by the ComposeFilter
        unimplemented!()
    }

    fn properties(&self, inprops: FstProperties) -> FstProperties {
        let oprops = self.filter.properties(inprops);
        if self.lookahead_output() {
            oprops & FstProperties::o_label_invariant_properties()
        } else {
            oprops & FstProperties::i_label_invariant_properties()
        }
    }
}

impl<W, F1, F2, B1, B2, M1, M2, CF, SMT> PushLabelsComposeFilter<W, F1, F2, B1, B2, M1, M2, CF, SMT>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug + Clone,
    B2: Borrow<F2> + Debug + Clone,
    M1: LookaheadMatcher<W, F1, B1>,
    M2: LookaheadMatcher<W, F2, B2>,
    CF: LookAheadComposeFilterTrait<W, F1, F2, B1, B2, M1, M2>,
    SMT: MatchTypeTrait,
{
    // Consumes an already pushed label.
    fn pushed_label_filter_tr(
        &self,
        arca: &mut Tr<W>,
        arcb: &mut Tr<W>,
        flabel: Label,
    ) -> Result<
        <Self as ComposeFilter<
            W,
            F1,
            F2,
            B1,
            B2,
            MultiEpsMatcher<W, F1, B1, M1>,
            MultiEpsMatcher<W, F2, B2, M2>,
        >>::FS,
    > {
        let labela = if self.lookahead_output() {
            &mut arca.olabel
        } else {
            &mut arca.ilabel
        };
        let labelb = if self.lookahead_output() {
            arcb.ilabel
        } else {
            arcb.olabel
        };

        let res = if labelb != NO_LABEL {
            FilterState::new_no_state()
        } else if *labela == flabel {
            *labela = EPS_LABEL;
            self.start()
        } else if *labela == EPS_LABEL {
            if self.ntrsa == 1
                || match self.selector() {
                    Selector::Fst1Matcher2 => {
                        let matcher = self.filter.matcher2();
                        matcher.lookahead_label(arca.nextstate, flabel)?
                    }
                    Selector::Fst2Matcher1 => {
                        let matcher = self.filter.matcher1();
                        matcher.lookahead_label(arca.nextstate, flabel)?
                    }
                }
            {
                self.fs.clone()
            } else {
                FilterState::new_no_state()
            }
        } else {
            FilterState::new_no_state()
        };
        Ok(res)
    }

    // Pushes a label forward when possible.
    fn push_label_filter_tr(
        &self,
        arca: &mut Tr<W>,
        arcb: &mut Tr<W>,
        fs1: &CF::FS,
    ) -> Result<
        <Self as ComposeFilter<
            W,
            F1,
            F2,
            B1,
            B2,
            MultiEpsMatcher<W, F1, B1, M1>,
            MultiEpsMatcher<W, F2, B2, M2>,
        >>::FS,
    > {
        let labela = if self.lookahead_output() {
            &mut arca.olabel
        } else {
            &mut arca.ilabel
        };
        let labelb = if self.lookahead_output() {
            arcb.olabel
        } else {
            arcb.ilabel
        };

        if labelb != EPS_LABEL {
            return Ok(FilterState::new((fs1.clone(), FilterState::new(NO_LABEL))));
        }

        if *labela != EPS_LABEL
            && self
                .lookahead_flags()
                .contains(MatcherFlags::LOOKAHEAD_NON_EPSILON_PREFIX)
        {
            return Ok(FilterState::new((fs1.clone(), FilterState::new(NO_LABEL))));
        }

        let mut larc = Tr::new(NO_LABEL, NO_LABEL, W::zero(), NO_STATE_ID);
        let la_matcher_data = self.filter.lookahead_matcher_data().unwrap();
        let b = match self.selector() {
            Selector::Fst1Matcher2 => {
                let matcher = self.filter.matcher2();
                matcher.lookahead_prefix(&mut larc, la_matcher_data)
            }
            Selector::Fst2Matcher1 => {
                let matcher = self.filter.matcher1();
                matcher.lookahead_prefix(&mut larc, la_matcher_data)
            }
        };

        if b {
            *labela = if self.lookahead_output() {
                larc.ilabel
            } else {
                larc.olabel
            };
            arcb.ilabel = larc.ilabel;
            arcb.olabel = larc.olabel;
            arcb.weight.times_assign(&larc.weight)?;
            arcb.nextstate = larc.nextstate;
            Ok(FilterState::new((fs1.clone(), FilterState::new(*labela))))
        } else {
            Ok(FilterState::new((fs1.clone(), FilterState::new(NO_LABEL))))
        }
    }

    fn lookahead_output(&self) -> bool {
        self.filter.lookahead_output()
    }

    fn lookahead_tr(&self) -> bool {
        self.filter.lookahead_tr()
    }

    fn lookahead_flags(&self) -> MatcherFlags {
        self.filter.lookahead_flags()
    }

    fn selector(&self) -> &Selector {
        self.filter.selector()
    }
}
