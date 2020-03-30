use std::cell::RefCell;
use std::rc::Rc;

use failure::Fallible;
use superslice::Ext;

use crate::{Arc, EPS_LABEL, Label, NO_LABEL, StateId};
use crate::algorithms::lookahead_matchers::LookaheadMatcher;
use crate::algorithms::matchers::{IterItemMatcher, Matcher, MatcherFlags, MatchType};
use crate::fst_properties::FstProperties;
use crate::fst_traits::{CoreFst, ExpandedFst};
use crate::semirings::Semiring;

#[derive(Debug, Clone, PartialEq)]
pub struct SortedMatcher<'fst, F: ExpandedFst> {
    fst: &'fst F,
    match_type: MatchType,
}

impl<'fst, W: Semiring + 'fst, F: ExpandedFst<W = W>> Matcher<'fst, W> for SortedMatcher<'fst, F> {
    type F = F;
    type Iter = IteratorSortedMatcher<'fst, W>;

    fn new(fst: &'fst F, match_type: MatchType) -> Fallible<Self> {
        Ok(Self { fst, match_type })
    }

    fn iter(&self, state: usize, label: usize) -> Fallible<Self::Iter> {
        Ok(IteratorSortedMatcher::new(
            self.fst.arcs_iter(state)?.collect(),
            label,
            self.match_type,
        ))
    }

    fn final_weight(&self, state: usize) -> Fallible<Option<&'fst W>> {
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

    fn priority(&self, state: StateId) -> Fallible<usize> {
        self.fst.num_arcs(state)
    }

    fn fst(&self) -> &'fst Self::F {
        self.fst
    }
}

#[derive(Clone)]
pub struct IteratorSortedMatcher<'fst, W: Semiring> {
    arcs: Vec<&'fst Arc<W>>,
    match_label: Label,
    pos: usize,
    current_loop: bool,
    match_type: MatchType,
}

impl<'fst, W: Semiring> IteratorSortedMatcher<'fst, W> {
    pub fn new(arcs: Vec<&'fst Arc<W>>, match_label: Label, match_type: MatchType) -> Self {
        // If we have to match epsilon, an epsilon loop is added
        let current_loop = match_label == EPS_LABEL;

        // NoLabel matches any non-consuming transitions, e.g., epsilon
        // transitions, which do not require a matching symbol.
        let match_label = if match_label == NO_LABEL {
            EPS_LABEL
        } else {
            match_label
        };

        // When matching epsilon, the first arc is supposed to be labeled as such
        let pos = if current_loop {
            0
        } else {
            match match_type {
                MatchType::MatchInput => arcs.lower_bound_by(|x| x.ilabel.cmp(&match_label)),
                MatchType::MatchOutput => arcs.lower_bound_by(|x| x.olabel.cmp(&match_label)),
                _ => panic!("Shouldn't happen : {:?}", match_type),
            }
        };

        Self {
            arcs,
            match_label,
            pos,
            current_loop,
            match_type,
        }
    }

    fn get_label(&self, arc: &Arc<W>) -> Label {
        match self.match_type {
            MatchType::MatchInput => arc.ilabel,
            MatchType::MatchOutput => arc.olabel,
            _ => panic!("Shouldn't happen : {:?}", self.match_type),
        }
    }
}

impl<'fst, W: Semiring> Iterator for IteratorSortedMatcher<'fst, W> {
    type Item = IterItemMatcher<'fst, W>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_loop {
            self.current_loop = false;
            return Some(IterItemMatcher::EpsLoop);
        }
        if let Some(arc) = self.arcs.get(self.pos) {
            if self.get_label(arc) == self.match_label {
                self.pos += 1;
                Some(IterItemMatcher::Arc(arc))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<'fst, F: ExpandedFst + 'fst> LookaheadMatcher<'fst, F::W> for SortedMatcher<'fst, F> {
    type MatcherData = ();

    fn data(&self) -> Option<&Rc<RefCell<Self::MatcherData>>> {
        unreachable!()
    }

    fn new_with_data(
        _fst: &'fst Self::F,
        _match_type: MatchType,
        _data: Option<Rc<RefCell<Self::MatcherData>>>,
    ) -> Fallible<Self>
    where
        Self: std::marker::Sized,
    {
        unreachable!()
    }

    fn create_data(_fst: &Self::F, _match_type: MatchType) -> Option<Rc<RefCell<Self::MatcherData>>> {
        unreachable!()
    }

    fn init_lookahead_fst<LF: ExpandedFst<W = F::W>>(&mut self, _lfst: &LF) -> Fallible<()> {
        unreachable!()
    }

    fn lookahead_fst<LF: ExpandedFst<W = F::W>>(
        &mut self,
        _matcher_state: usize,
        _lfst: &LF,
        _lfst_state: usize,
    ) -> Fallible<bool> {
        unreachable!()
    }

    fn lookahead_label(&self, _state: usize, _label: usize) -> Fallible<bool> {
        unreachable!()
    }

    fn lookahead_prefix(&self, _arc: &mut Arc<<F as CoreFst>::W>) -> bool {
        unreachable!()
    }

    fn lookahead_weight(&self) -> &<F as CoreFst>::W {
        unreachable!()
    }

    fn prefix_arc(&self) -> &Arc<<F as CoreFst>::W> {
        unreachable!()
    }

    fn prefix_arc_mut(&mut self) -> &mut Arc<<F as CoreFst>::W> {
        unreachable!()
    }

    fn lookahead_weight_mut(&mut self) -> &mut <F as CoreFst>::W {
        unreachable!()
    }
}
