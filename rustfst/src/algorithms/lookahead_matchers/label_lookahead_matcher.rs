use failure::Fallible;

use crate::algorithms::lookahead_matchers::label_reachable::{LabelReachable, LabelReachableData};
use crate::algorithms::lookahead_matchers::LookaheadMatcher;
use crate::algorithms::matchers::{IterItemMatcher, MatchType, Matcher, MatcherFlags};
use crate::fst_traits::{CoreFst, Fst};
use crate::semirings::Semiring;
use crate::{Arc, Label, StateId, EPS_LABEL, NO_LABEL, NO_STATE_ID};
use unsafe_unwrap::UnsafeUnwrap;

#[derive(Debug)]
struct LabelLookAheadMatcher<'fst, W: Semiring, M: Matcher<'fst, W>> {
    // matcher fst
    fst: &'fst M::F,
    matcher: M,
    lookahead_weight: W,
    prefix_arc: Arc<W>,

    // Flags to customize the behaviour
    flags: MatcherFlags,
    reachable: Option<LabelReachable>,
}

impl<'fst, W: Semiring, M: Matcher<'fst, W>> LabelLookAheadMatcher<'fst, W, M> {
    pub fn new_with_flags(
        fst: &'fst M::F,
        match_type: MatchType,
        matcher_flags: MatcherFlags,
        data: Option<&LabelReachableData>,
    ) -> Fallible<Self> {
        if !matcher_flags.contains(
            MatcherFlags::INPUT_LOOKAHEAD_MATCHER | MatcherFlags::OUTPUT_LOOKAHEAD_MATCHER,
        ) {
            bail!(
                "LabelLookAheadMatcher : Bad Matcher flags : {:?}",
                matcher_flags
            )
        }
        let reach_input = match_type == MatchType::MatchInput;

        let mut reachable = None;
        if let Some(d) = data {
            if reach_input == d.reach_input() {
                reachable = Some(LabelReachable::new_from_data(d.clone()));
            }
        } else if (reach_input && matcher_flags.contains(MatcherFlags::INPUT_LOOKAHEAD_MATCHER))
            || (!reach_input && matcher_flags.contains(MatcherFlags::OUTPUT_LOOKAHEAD_MATCHER))
        {
            reachable = Some(LabelReachable::new(fst, reach_input)?)
        }

        Ok(Self {
            fst,
            matcher: M::new(fst, match_type)?,
            flags: matcher_flags,
            prefix_arc: Arc::new(0, 0, W::one(), NO_STATE_ID),
            lookahead_weight: W::one(),
            reachable,
        })
    }
}

impl<'fst, W: Semiring, M: Matcher<'fst, W>> Matcher<'fst, W>
    for LabelLookAheadMatcher<'fst, W, M>
{
    type F = M::F;
    type Iter = M::Iter;

    fn new(fst: &'fst Self::F, match_type: MatchType) -> Fallible<Self> {
        unimplemented!()
        // let flags = MatcherFlags::LOOKAHEAD_EPSILONS
        //     | MatcherFlags::LOOKAHEAD_WEIGHT
        //     | MatcherFlags::LOOKAHEAD_PREFIX
        //     | MatcherFlags::LOOKAHEAD_NON_EPSILON_PREFIX;
        // Ok(Self {
        //     fst,
        //     matcher: M::new(fst, match_type)?,
        //     flags,
        //     prefix_arc: Arc::new(0, 0, W::one(), NO_STATE_ID),
        //     lookahead_weight: W::one(),
        // })
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
        unimplemented!()
    }

    fn priority(&self, state: usize) -> Fallible<usize> {
        self.matcher.priority(state)
    }
}

impl<'fst, W: Semiring, M: Matcher<'fst, W>> LookaheadMatcher<'fst, W>
    for LabelLookAheadMatcher<'fst, W, M>
{
    fn lookahead_fst<LF: Fst<W = W>>(
        &mut self,
        matcher_state: usize,
        lfst: &LF,
        lfst_state: usize,
    ) -> Fallible<bool> {
        unimplemented!()
    }

    fn lookahead_label(&self, state: usize, label: usize) -> Fallible<bool> {
        unimplemented!()
    }

    fn lookahead_prefix(&self, arc: &mut Arc<W>) -> bool {
        unimplemented!()
    }

    fn lookahead_weight(&self) -> &W {
        unimplemented!()
    }

    fn prefix_arc(&self) -> &Arc<W> {
        unimplemented!()
    }

    fn prefix_arc_mut(&mut self) -> &mut Arc<W> {
        unimplemented!()
    }

    fn lookahead_weight_mut(&mut self) -> &mut W {
        unimplemented!()
    }
}
