use failure::Fallible;

use crate::algorithms::compose_filters::ComposeFilter;
use crate::algorithms::filter_states::{CharFilterState, FilterState};
use crate::algorithms::matchers::{MatchType, Matcher};
use crate::fst_traits::{CoreFst, Fst};
use crate::{Arc, StateId, EPS_LABEL, NO_LABEL, NO_STATE_ID};

pub struct MatchComposeFilter<'fst, F1, F2, M1, M2> {
    fst1: &'fst F1,
    fst2: &'fst F2,
    matcher1: M1,
    matcher2: M2,
    /// Current fst1 state
    s1: StateId,
    /// Current fst2 state
    s2: StateId,
    /// Current filter state
    fs: CharFilterState,
    /// Only epsilons (and non-final) leaving s1 ?
    alleps1: bool,
    /// Only epsilons (and non-final) leaving s2 ?
    alleps2: bool,
    /// No epsilons leaving s1 ?
    noeps1: bool,
    /// No epsilons leaving s2 ?
    noeps2: bool,
}

impl<
        'matcher,
        'fst: 'matcher,
        F1: Fst + 'fst,
        F2: Fst<W = F1::W> + 'fst,
        M1: Matcher<'matcher, 'fst, F1>,
        M2: Matcher<'matcher, 'fst, F2>,
    > ComposeFilter<'matcher, 'fst, F1, F2> for MatchComposeFilter<'fst, F1, F2, M1, M2>
{
    type M1 = M1;
    type M2 = M2;
    type FS = CharFilterState;

    fn new(fst1: &'fst F1, fst2: &'fst F2) -> Fallible<Self> {
        Ok(Self {
            fst1,
            fst2,
            matcher1: Self::M1::new(fst1, MatchType::MatchOutput)?,
            matcher2: Self::M2::new(fst2, MatchType::MatchInput)?,
            s1: NO_STATE_ID,
            s2: NO_STATE_ID,
            fs: Self::FS::default(),
            alleps1: false,
            alleps2: false,
            noeps1: false,
            noeps2: false,
        })
    }

    fn start(&self) -> Self::FS {
        Self::FS::new(0)
    }

    fn set_state(&mut self, s1: usize, s2: usize, filter_state: &Self::FS) {
        if !(self.s1 == s1 && self.s2 == s2 && &self.fs == filter_state) {
            self.s1 = s1;
            self.s2 = s2;
            self.fs = filter_state.clone();

            let na1 = self.fst1.num_arcs(s1).unwrap();
            let na2 = self.fst2.num_arcs(s2).unwrap();

            let ne1 = self.fst1.num_output_epsilons(s1).unwrap();
            let ne2 = self.fst2.num_input_epsilons(s2).unwrap();

            let f1 = self.fst1.is_final(s1).unwrap();
            let f2 = self.fst2.is_final(s2).unwrap();

            self.alleps1 = na1 == ne1 && !f1;
            self.alleps2 = na2 == ne2 && !f2;

            self.noeps1 = ne1 == 0;
            self.noeps2 = ne2 == 0;
        }
    }

    fn filter_arc(
        &self,
        arc1: &mut Arc<<F1 as CoreFst>::W>,
        arc2: &mut Arc<<F2 as CoreFst>::W>,
    ) -> Option<Self::FS> {
        if arc2.ilabel == NO_LABEL {
            // EPSILON in FST1
            if self.fs == Self::FS::new(0) {
                if self.noeps2 {
                    Some(Self::FS::new(0))
                } else if self.alleps2 {
                    None
                } else {
                    Some(Self::FS::new(1))
                }
            } else {
                if self.fs == Self::FS::new(1) {
                    Some(Self::FS::new(1))
                } else {
                    None
                }
            }
        } else if arc1.olabel == NO_LABEL {
            // Epsilon in FST2
            if self.fs == Self::FS::new(0) {
                if self.noeps1 {
                    Some(Self::FS::new(0))
                } else if self.alleps1 {
                    None
                } else {
                    Some(Self::FS::new(2))
                }
            } else {
                if self.fs == Self::FS::new(2) {
                    Some(Self::FS::new(2))
                } else {
                    None
                }
            }
        } else if arc1.olabel == EPS_LABEL {
            // Epsilon in both
            if self.fs == Self::FS::new(0) {
                Some(Self::FS::new(0))
            } else {
                None
            }
        } else {
            // Both are non-epsilons
            Some(Self::FS::new(0))
        }
    }

    fn filter_final(&self, _w1: &mut <F1 as CoreFst>::W, _w2: &mut <F2 as CoreFst>::W) {}

    fn matcher1(&mut self) -> &mut Self::M1 {
        &mut self.matcher1
    }

    fn matcher2(&mut self) -> &mut Self::M2 {
        &mut self.matcher2
    }
}
