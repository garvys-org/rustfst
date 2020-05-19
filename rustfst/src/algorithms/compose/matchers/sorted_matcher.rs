use std::sync::Arc;

use anyhow::Result;
use superslice::Ext;

use crate::algorithms::compose::lookahead_matchers::{LookAheadMatcherData, LookaheadMatcher};
use crate::algorithms::compose::matchers::{IterItemMatcher, MatchType, Matcher, MatcherFlags};
use crate::fst_properties::FstProperties;
use crate::fst_traits::{CoreFst, ExpandedFst};
use crate::semirings::Semiring;
use crate::{Label, StateId, Tr, Trs, EPS_LABEL, NO_LABEL};
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq)]
pub struct SortedMatcher<W: Semiring, F: ExpandedFst<W>> {
    fst: Arc<F>,
    match_type: MatchType,
    w: PhantomData<W>,
}

impl<W: Semiring, F: ExpandedFst<W>> Matcher<W> for SortedMatcher<W, F> {
    type F = F;
    type Iter = IteratorSortedMatcher<W, F::TRS>;

    fn new(fst: Arc<F>, match_type: MatchType) -> Result<Self> {
        Ok(Self {
            fst,
            match_type,
            w: PhantomData,
        })
    }

    fn iter(&self, state: usize, label: usize) -> Result<Self::Iter> {
        Ok(IteratorSortedMatcher::new(
            self.fst.get_trs(state)?,
            label,
            self.match_type,
        ))
    }

    fn final_weight(&self, state: usize) -> Result<Option<W>> {
        self.fst.final_weight(state)
    }

    fn match_type(&self) -> MatchType {
        if self.match_type == MatchType::MatchNone {
            return self.match_type;
        }
        let true_prop = if self.match_type == MatchType::MatchInput {
            FstProperties::I_LABEL_SORTED
        } else {
            FstProperties::O_LABEL_SORTED
        };

        let false_prop = if self.match_type == MatchType::MatchInput {
            FstProperties::NOT_I_LABEL_SORTED
        } else {
            FstProperties::NOT_O_LABEL_SORTED
        };

        let props = self.fst.properties().unwrap();

        if props.contains(true_prop) {
            self.match_type
        } else if props.contains(false_prop) {
            MatchType::MatchNone
        } else {
            MatchType::MatchUnknown
        }
    }

    fn flags(&self) -> MatcherFlags {
        MatcherFlags::empty()
    }

    fn priority(&self, state: StateId) -> Result<usize> {
        self.fst.num_trs(state)
    }

    fn fst(&self) -> &Arc<Self::F> {
        &self.fst
    }
}

pub struct IteratorSortedMatcher<W: Semiring, T: Trs<W>> {
    trs: T,
    match_label: Label,
    pos: usize,
    current_loop: bool,
    match_type: MatchType,
    w: PhantomData<W>,
}

// Clone that doesn't copy the data inside Trs, only the Arc
impl<W: Semiring, T: Trs<W>> Clone for IteratorSortedMatcher<W, T> {
    fn clone(&self) -> Self {
        Self {
            trs: self.trs.shallow_clone(),
            match_label: self.match_label,
            pos: self.pos,
            current_loop: self.current_loop,
            match_type: self.match_type,
            w: PhantomData,
        }
    }
}

impl<W: Semiring, T: Trs<W>> IteratorSortedMatcher<W, T> {
    pub fn new(trs: T, match_label: Label, match_type: MatchType) -> Self {
        // If we have to match epsilon, an epsilon loop is added
        let current_loop = match_label == EPS_LABEL;

        // NoLabel matches any non-consuming transitions, e.g., epsilon
        // transitions, which do not require a matching symbol.
        let match_label = if match_label == NO_LABEL {
            EPS_LABEL
        } else {
            match_label
        };

        // When matching epsilon, the first transition is supposed to be labeled as such
        let pos = if current_loop {
            0
        } else {
            match match_type {
                MatchType::MatchInput => trs.lower_bound_by(|x| x.ilabel.cmp(&match_label)),
                MatchType::MatchOutput => trs.lower_bound_by(|x| x.olabel.cmp(&match_label)),
                _ => panic!("Shouldn't happen : {:?}", match_type),
            }
        };

        Self {
            trs,
            match_label,
            pos,
            current_loop,
            match_type,
            w: PhantomData,
        }
    }

    fn get_label(&self, tr: &Tr<W>) -> Label {
        match self.match_type {
            MatchType::MatchInput => tr.ilabel,
            MatchType::MatchOutput => tr.olabel,
            _ => panic!("Shouldn't happen : {:?}", self.match_type),
        }
    }
}

impl<W: Semiring, T: Trs<W>> Iterator for IteratorSortedMatcher<W, T> {
    type Item = IterItemMatcher<W>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_loop {
            self.current_loop = false;
            return Some(IterItemMatcher::EpsLoop);
        }
        if let Some(tr) = self.trs.get(self.pos) {
            if self.get_label(tr) == self.match_label {
                self.pos += 1;
                Some(IterItemMatcher::Tr(tr.clone()))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<W: Semiring, F: ExpandedFst<W>> LookaheadMatcher<W> for SortedMatcher<W, F> {
    type MatcherData = ();

    fn data(&self) -> Option<&Arc<Self::MatcherData>> {
        unreachable!()
    }

    fn new_with_data(
        _fst: Arc<Self::F>,
        _match_type: MatchType,
        _data: Option<Arc<Self::MatcherData>>,
    ) -> Result<Self>
    where
        Self: std::marker::Sized,
    {
        unreachable!()
    }

    fn create_data<G: ExpandedFst<W>>(
        _fst: &G,
        _match_type: MatchType,
    ) -> Result<Option<Self::MatcherData>> {
        unreachable!()
    }

    fn init_lookahead_fst<LF: ExpandedFst<W>>(&mut self, _lfst: &Arc<LF>) -> Result<()> {
        unreachable!()
    }

    fn lookahead_fst<LF: ExpandedFst<W>>(
        &self,
        _matcher_state: usize,
        _lfst: &Arc<LF>,
        _lfst_state: usize,
    ) -> Result<Option<LookAheadMatcherData<W>>> {
        unreachable!()
    }

    fn lookahead_label(&self, _state: usize, _label: usize) -> Result<bool> {
        unreachable!()
    }

    fn lookahead_prefix(
        &self,
        _tr: &mut Tr<W>,
        _la_matcher_data: &LookAheadMatcherData<W>,
    ) -> bool {
        unreachable!()
    }
}
