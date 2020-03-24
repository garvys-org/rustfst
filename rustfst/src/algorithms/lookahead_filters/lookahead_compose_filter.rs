use crate::algorithms::compose_filters::ComposeFilter;
use crate::algorithms::filter_states::FilterState;
use crate::algorithms::lookahead_filters::lookahead_match_type;
use crate::algorithms::lookahead_filters::lookahead_selector::{selector_match_input, selector_match_output, LookAheadSelector, MatchTypeTrait, selector};
use crate::algorithms::matchers::MatcherFlags;
use crate::algorithms::matchers::{MatchType, Matcher};
use crate::semirings::Semiring;
use crate::{Arc, EPS_LABEL};
use failure::Fallible;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use crate::algorithms::lookahead_matchers::LookaheadMatcher;

#[derive(Debug)]
struct LookAheadComposeFilter<W, CF, SMT> {
    filter: CF,
    lookahead_type: MatchType,
    flags: MatcherFlags,
    lookahead_arc: bool,
    smt: PhantomData<SMT>,
    w: PhantomData<W>,
}

impl<
        'fst1,
        'fst2,
        W: Semiring + 'fst1 + 'fst2,
        CF: ComposeFilter<'fst1, 'fst2, W>,
        SMT: MatchTypeTrait,
    > LookAheadComposeFilter<W, CF, SMT>
    where CF::M1: LookaheadMatcher<'fst1, W>, CF::M2: LookaheadMatcher<'fst2, W>
{
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

    fn lookahead_filter_arc(
        &mut self,
        arca: &mut Arc<W>,
        arcb: &mut Arc<W>,
        fs: &CF::FS,
    ) -> Fallible<CF::FS> {
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
        self.lookahead_arc = true;

        let fn1 = |selector: LookAheadSelector<<CF::M1 as Matcher<'fst1, W>>::F, CF::M2>| {
            selector.matcher.borrow_mut().lookahead_fst(arca.nextstate, selector.fst, arcb.nextstate)
        };

        let fn2 = |selector: LookAheadSelector<<CF::M2 as Matcher<'fst2, W>>::F, CF::M1>| {
            selector.matcher.borrow_mut().lookahead_fst(arca.nextstate, selector.fst, arcb.nextstate)
        };

        let res = selector(self.matcher1(), self.matcher2(), SMT::match_type(), self.lookahead_type, fn1, fn2)?;
        if res {
            Ok(fs.clone())
        } else {
            Ok(CF::FS::new_no_state())
        }
    }
}

impl<
        'fst1,
        'fst2,
        W: Semiring + 'fst1 + 'fst2,
        CF: ComposeFilter<'fst1, 'fst2, W>,
        SMT: MatchTypeTrait,
    > ComposeFilter<'fst1, 'fst2, W> for LookAheadComposeFilter<W, CF, SMT>
where CF::M1: LookaheadMatcher<'fst1, W>, CF::M2: LookaheadMatcher<'fst2, W>
{
    type M1 = CF::M1;
    type M2 = CF::M2;
    type FS = CF::FS;

    fn new<IM1: Into<Option<Self::M1>>, IM2: Into<Option<Self::M2>>>(
        fst1: &'fst1 <Self::M1 as Matcher<'fst1, W>>::F,
        fst2: &'fst2 <Self::M2 as Matcher<'fst2, W>>::F,
        m1: IM1,
        m2: IM2,
    ) -> Fallible<Self> {
        let filter = CF::new(fst1, fst2, m1, m2)?;
        let lookahead_type = if SMT::match_type() == MatchType::MatchBoth {
            unimplemented!()
        } else {
            SMT::match_type()
        };

        let flags = if lookahead_type == MatchType::MatchOutput {
            filter.matcher1().borrow().flags()
        } else {
            filter.matcher2().borrow().flags()
        };

        if lookahead_type == MatchType::MatchNone {
            bail!(
                "LookAheadComposeFilter: 1st argument cannot match/look-ahead on output \
            labels and 2nd argument cannot match/look-ahead on input labels"
            )
        }

        panic!("Selector");
        // Selector logic.
        match SMT::match_type() {
            MatchType::MatchInput => {
                selector_match_input(filter.matcher1(), filter.matcher2());
            }
            MatchType::MatchOutput => {
                selector_match_output(filter.matcher1(), filter.matcher2());
            }
            _ => {
                if lookahead_type == MatchType::MatchOutput {
                    selector_match_output(filter.matcher1(), filter.matcher2());
                } else {
                    selector_match_input(filter.matcher1(), filter.matcher2());
                }
            }
        };
        // let selector = LookAheadSelector::new(filter.matcher1(), filter.matcher2(), lookahead_type);
        Ok(Self {
            filter,
            lookahead_type,
            flags,
            smt: PhantomData,
            lookahead_arc: false,
            w: PhantomData,
        })
    }

    fn start(&self) -> Self::FS {
        self.filter.start()
    }

    fn set_state(&mut self, s1: usize, s2: usize, filter_state: &Self::FS) {
        self.filter.set_state(s1, s2, filter_state)
    }

    fn filter_arc(&mut self, arc1: &mut Arc<W>, arc2: &mut Arc<W>) -> Self::FS {
        self.lookahead_arc = false;
        let fs = self.filter.filter_arc(arc1, arc2);
        if fs == CF::FS::new_no_state() {
            return CF::FS::new_no_state();
        }
        if self.lookahead_output() {
            self.lookahead_filter_arc(arc1, arc2, &fs).unwrap()
        } else {
            self.lookahead_filter_arc(arc2, arc1, &fs).unwrap()
        }
    }

    fn filter_final(&self, w1: &mut W, w2: &mut W) {
        self.filter.filter_final(w1, w2)
    }

    fn matcher1(&self) -> Rc<RefCell<Self::M1>> {
        self.filter.matcher1()
    }

    fn matcher2(&self) -> Rc<RefCell<Self::M2>> {
        self.filter.matcher2()
    }
}
