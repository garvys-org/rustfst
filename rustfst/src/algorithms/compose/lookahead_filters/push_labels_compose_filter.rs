use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use failure::Fallible;

use crate::algorithms::compose::compose_filters::ComposeFilter;
use crate::algorithms::compose::filter_states::{FilterState, IntegerFilterState, PairFilterState};
use crate::algorithms::compose::lookahead_filters::lookahead_selector::MatchTypeTrait;
use crate::algorithms::compose::lookahead_filters::lookahead_selector::Selector;
use crate::algorithms::compose::lookahead_filters::LookAheadComposeFilterTrait;
use crate::algorithms::compose::lookahead_matchers::LookaheadMatcher;
use crate::algorithms::compose::matchers::MatcherFlags;
use crate::algorithms::compose::matchers::{MatchType, Matcher};
use crate::algorithms::compose::matchers::{MultiEpsMatcher, MultiEpsMatcherFlags};
use crate::fst_traits::CoreFst;
use crate::semirings::Semiring;
use crate::{Arc, Label, EPS_LABEL, NO_LABEL, NO_STATE_ID};

#[derive(Debug, Clone)]
pub struct PushLabelsComposeFilter<
    'fst1,
    'fst2,
    W: Semiring + 'fst1 + 'fst2,
    CF: LookAheadComposeFilterTrait<'fst1, 'fst2, W>,
    SMT: MatchTypeTrait,
> where
    CF::M1: LookaheadMatcher<'fst1, W>,
    CF::M2: LookaheadMatcher<'fst2, W>,
{
    fst1: &'fst1 <CF::M1 as Matcher<'fst1, W>>::F,
    fst2: &'fst2 <CF::M2 as Matcher<'fst2, W>>::F,
    matcher1: Rc<RefCell<MultiEpsMatcher<W, CF::M1>>>,
    matcher2: Rc<RefCell<MultiEpsMatcher<W, CF::M2>>>,
    filter: CF,
    fs: PairFilterState<CF::FS, IntegerFilterState>,
    smt: PhantomData<SMT>,
    narcsa: usize,
}

impl<
        'fst1,
        'fst2,
        W: Semiring + 'fst1 + 'fst2,
        CF: LookAheadComposeFilterTrait<'fst1, 'fst2, W>,
        SMT: MatchTypeTrait,
    > ComposeFilter<'fst1, 'fst2, W> for PushLabelsComposeFilter<'fst1, 'fst2, W, CF, SMT>
where
    CF::M1: LookaheadMatcher<'fst1, W>,
    CF::M2: LookaheadMatcher<'fst2, W>,
{
    type M1 = MultiEpsMatcher<W, CF::M1>;
    type M2 = MultiEpsMatcher<W, CF::M2>;
    type FS = PairFilterState<CF::FS, IntegerFilterState>;

    fn new<IM1: Into<Option<Rc<RefCell<Self::M1>>>>, IM2: Into<Option<Rc<RefCell<Self::M2>>>>>(
        _fst1: &'fst1 <Self::M1 as Matcher<'fst1, W>>::F,
        _fst2: &'fst2 <Self::M2 as Matcher<'fst2, W>>::F,
        _m1: IM1,
        _m2: IM2,
    ) -> Fallible<Self> {
        unimplemented!()
    }

    fn start(&self) -> Self::FS {
        PairFilterState::new((self.filter.start(), FilterState::new(NO_LABEL)))
    }

    fn set_state(&mut self, s1: usize, s2: usize, filter_state: &Self::FS) -> Fallible<()> {
        self.fs = filter_state.clone();
        self.filter.set_state(s1, s2, filter_state.state1())?;
        if !self
            .filter
            .lookahead_flags()
            .contains(MatcherFlags::LOOKAHEAD_PREFIX)
        {
            return Ok(());
        }
        self.narcsa = if self.lookahead_output() {
            self.fst1.num_arcs(s1)?
        } else {
            self.fst2.num_arcs(s2)?
        };
        let fs2 = filter_state.state2();
        let flabel = fs2.state();
        self.matcher1().borrow_mut().clear_multi_eps_labels();
        self.matcher2().borrow_mut().clear_multi_eps_labels();
        if *flabel != NO_LABEL {
            self.matcher1().borrow_mut().add_multi_eps_label(*flabel)?;
            self.matcher2().borrow_mut().add_multi_eps_label(*flabel)?;
        }
        Ok(())
    }

    fn filter_arc(&mut self, arc1: &mut Arc<W>, arc2: &mut Arc<W>) -> Fallible<Self::FS> {
        if !self
            .lookahead_flags()
            .contains(MatcherFlags::LOOKAHEAD_PREFIX)
        {
            return Ok(FilterState::new((
                self.filter.filter_arc(arc1, arc2)?,
                FilterState::new(NO_LABEL),
            )));
        }
        let fs2 = self.fs.state2();
        let flabel = fs2.state();
        if *flabel != NO_LABEL {
            if self.lookahead_output() {
                return self.pushed_label_filter_arc(arc1, arc2, *flabel);
            } else {
                return self.pushed_label_filter_arc(arc2, arc1, *flabel);
            }
        }
        let fs1 = self.filter.filter_arc(arc1, arc2)?;
        if fs1 == FilterState::new_no_state() {
            return Ok(FilterState::new_no_state());
        }
        if !self.lookahead_arc() {
            return Ok(FilterState::new((fs1, FilterState::new(NO_LABEL))));
        }
        if self.lookahead_output() {
            self.push_label_filter_arc(arc1, arc2, &fs1)
        } else {
            self.push_label_filter_arc(arc2, arc1, &fs1)
        }
    }

    fn filter_final(&self, w1: &mut W, w2: &mut W) -> Fallible<()> {
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

    fn matcher1(&self) -> Rc<RefCell<Self::M1>> {
        Rc::clone(&self.matcher1)
    }

    fn matcher2(&self) -> Rc<RefCell<Self::M2>> {
        Rc::clone(&self.matcher2)
    }
}

impl<
        'fst1,
        'fst2,
        W: Semiring + 'fst1 + 'fst2,
        CF: LookAheadComposeFilterTrait<'fst1, 'fst2, W>,
        SMT: MatchTypeTrait,
    > PushLabelsComposeFilter<'fst1, 'fst2, W, CF, SMT>
where
    CF::M1: LookaheadMatcher<'fst1, W>,
    CF::M2: LookaheadMatcher<'fst2, W>,
{
    // Consumes an already pushed label.
    fn pushed_label_filter_arc(
        &self,
        arca: &mut Arc<W>,
        arcb: &mut Arc<W>,
        flabel: Label,
    ) -> Fallible<<Self as ComposeFilter<'fst1, 'fst2, W>>::FS> {
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
            if self.narcsa == 1 {
                self.fs.clone()
            } else {
                if match self.selector() {
                    Selector::MatchInput(s) => s
                        .matcher
                        .borrow_mut()
                        .lookahead_label(arca.nextstate, flabel)?,
                    Selector::MatchOutput(s) => s
                        .matcher
                        .borrow_mut()
                        .lookahead_label(arca.nextstate, flabel)?,
                } {
                    self.fs.clone()
                } else {
                    FilterState::new_no_state()
                }
            }
        } else {
            FilterState::new_no_state()
        };
        Ok(res)
    }

    // Pushes a label forward when possible.
    fn push_label_filter_arc(
        &self,
        arca: &mut Arc<W>,
        arcb: &mut Arc<W>,
        fs1: &CF::FS,
    ) -> Fallible<<Self as ComposeFilter<'fst1, 'fst2, W>>::FS> {
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

        let mut larc = Arc::new(NO_LABEL, NO_LABEL, W::zero(), NO_STATE_ID);

        let b = match self.selector() {
            Selector::MatchInput(s) => s.matcher.borrow().lookahead_prefix(&mut larc),
            Selector::MatchOutput(s) => s.matcher.borrow().lookahead_prefix(&mut larc),
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

    fn lookahead_arc(&self) -> bool {
        self.filter.lookahead_arc()
    }

    fn lookahead_flags(&self) -> MatcherFlags {
        self.filter.lookahead_flags()
    }

    fn selector(&self) -> &Selector<'fst1, 'fst2, W, CF::M1, CF::M2> {
        self.filter.selector()
    }

    pub fn new_2<IM1: Into<Option<Rc<RefCell<CF::M1>>>>, IM2: Into<Option<Rc<RefCell<CF::M2>>>>>(
        fst1: &'fst1 <<Self as ComposeFilter<'fst1, 'fst2, W>>::M1 as Matcher<'fst1, W>>::F,
        fst2: &'fst2 <<Self as ComposeFilter<'fst1, 'fst2, W>>::M2 as Matcher<'fst2, W>>::F,
        m1: IM1,
        m2: IM2,
    ) -> Fallible<Self> {
        let filter = CF::new(fst1, fst2, m1, m2)?;
        let fst1 = filter.matcher1().borrow().fst();
        let fst2 = filter.matcher2().borrow().fst();
        let matcher1 = Rc::new(RefCell::new(MultiEpsMatcher::new_with_opts(
            fst1,
            MatchType::MatchOutput,
            if filter.lookahead_output() {
                MultiEpsMatcherFlags::MULTI_EPS_LIST
            } else {
                MultiEpsMatcherFlags::MULTI_EPS_LOOP
            },
            filter.matcher1(),
        )?));
        let matcher2 = Rc::new(RefCell::new(MultiEpsMatcher::new_with_opts(
            fst2,
            MatchType::MatchInput,
            if filter.lookahead_output() {
                MultiEpsMatcherFlags::MULTI_EPS_LOOP
            } else {
                MultiEpsMatcherFlags::MULTI_EPS_LIST
            },
            filter.matcher2(),
        )?));
        Ok(Self {
            fs: FilterState::new_no_state(),
            fst1,
            fst2,
            matcher1,
            matcher2,
            filter,
            narcsa: 0,
            smt: PhantomData,
        })
    }
}
