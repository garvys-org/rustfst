use std::borrow::Borrow;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;
use superslice::Ext;

use crate::algorithms::compose::lookahead_matchers::{LookAheadMatcherData, LookaheadMatcher};
use crate::algorithms::compose::matchers::{IterItemMatcher, MatchType, Matcher, MatcherFlags};
use crate::fst_properties::FstProperties;
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{Label, StateId, Tr, Trs, EPS_LABEL, NO_LABEL};

#[derive(Debug, Clone, PartialEq)]
pub struct SortedMatcher<W, F, B>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F>,
{
    fst: B,
    match_type: MatchType,
    w: PhantomData<(W, F)>,
}

impl<W, F, B> Matcher<W, F, B> for SortedMatcher<W, F, B>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F> + Debug,
{
    type Iter = IteratorSortedMatcher<W, F::TRS>;

    fn new(fst: B, match_type: MatchType) -> Result<Self> {
        Ok(Self {
            fst,
            match_type,
            w: PhantomData,
        })
    }

    fn iter(&self, state: StateId, label: Label) -> Result<Self::Iter> {
        Ok(IteratorSortedMatcher::new(
            self.fst.borrow().get_trs(state)?,
            label,
            self.match_type,
        ))
    }

    fn final_weight(&self, state: StateId) -> Result<Option<W>> {
        self.fst.borrow().final_weight(state)
    }

    fn match_type(&self, test: bool) -> Result<MatchType> {
        if self.match_type == MatchType::MatchNone {
            return Ok(self.match_type);
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

        let props = if test {
            self.fst.borrow().properties_check(true_prop | false_prop)?
        } else {
            self.fst.borrow().properties()
        };

        if props.contains(true_prop) {
            Ok(self.match_type)
        } else if props.contains(false_prop) {
            Ok(MatchType::MatchNone)
        } else {
            Ok(MatchType::MatchUnknown)
        }
    }

    fn flags(&self) -> MatcherFlags {
        MatcherFlags::empty()
    }

    fn priority(&self, state: StateId) -> Result<usize> {
        self.fst.borrow().num_trs(state)
    }

    fn fst(&self) -> &B {
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

impl<W, F, B> LookaheadMatcher<W, F, B> for SortedMatcher<W, F, B>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F> + Debug,
{
    type MatcherData = ();

    fn data(&self) -> Option<&Arc<Self::MatcherData>> {
        unreachable!()
    }

    fn new_with_data(
        _fst: B,
        _match_type: MatchType,
        _data: Option<Arc<Self::MatcherData>>,
    ) -> Result<Self>
    where
        Self: std::marker::Sized,
    {
        unreachable!()
    }

    fn create_data<F2: Fst<W>, BF2: Borrow<F2>>(
        _fst: BF2,
        _match_type: MatchType,
    ) -> Result<Option<Self::MatcherData>> {
        unreachable!()
    }

    fn init_lookahead_fst<LF: Fst<W>, BLF: Borrow<LF>>(&mut self, _lfst: &BLF) -> Result<()> {
        unreachable!()
    }

    fn lookahead_fst<LF: Fst<W>, BLF: Borrow<LF>>(
        &self,
        _matcher_state: StateId,
        _lfst: &BLF,
        _lfst_state: StateId,
    ) -> Result<Option<LookAheadMatcherData<W>>> {
        unreachable!()
    }

    fn lookahead_label(&self, _state: StateId, _label: Label) -> Result<bool> {
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
