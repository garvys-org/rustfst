use crate::algorithms::compose_filters::ComposeFilter;
use crate::algorithms::matchers::{MatchType, Matcher};
use crate::semirings::Semiring;
use crate::Arc;
use failure::_core::cell::RefCell;
use std::rc::Rc;
use failure::Fallible;
use std::marker::PhantomData;
use crate::algorithms::lookahead_filters::lookahead_selector::{MatchTypeTrait, LookAheadSelector, selector_match_input, selector_match_output};
use crate::algorithms::lookahead_filters::lookahead_match_type;
use crate::algorithms::matchers::MatcherFlags;

#[derive(Debug)]
struct LookAheadComposeFilter<CF, SMT> {
    filter: CF,
    lookahead_type: MatchType,
    smt: PhantomData<SMT>,
    flags: MatcherFlags,
    lookahead_arc: bool,
}

impl<'fst1, 'fst2, W: Semiring + 'fst1 + 'fst2, CF: ComposeFilter<'fst1, 'fst2, W>, SMT: MatchTypeTrait>
    ComposeFilter<'fst1, 'fst2, W> for LookAheadComposeFilter<CF, SMT>
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
            bail!("LookAheadComposeFilter: 1st argument cannot match/look-ahead on output \
            labels and 2nd argument cannot match/look-ahead on input labels")
        }

        panic!("Selector");
        // Selector logic.
        match SMT::match_type(){
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
            filter, lookahead_type, flags, smt: PhantomData, lookahead_arc: false
        })

    }

    fn start(&self) -> Self::FS {
        self.filter.start()
    }

    fn set_state(&mut self, s1: usize, s2: usize, filter_state: &Self::FS) {
        self.filter.set_state(s1, s2, filter_state)
    }

    fn filter_arc(&self, arc1: &mut Arc<W>, arc2: &mut Arc<W>) -> Self::FS {
        unimplemented!()
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
