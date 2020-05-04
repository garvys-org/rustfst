use std::cell::RefCell;
use std::rc::Rc;

use anyhow::Result;
use superslice::Ext;

use crate::algorithms::compose::lookahead_matchers::LookaheadMatcher;
use crate::algorithms::compose::matchers::{IterItemMatcher, MatchType, Matcher, MatcherFlags};
use crate::fst_properties::FstProperties;
use crate::fst_traits::{CoreFst, ExpandedFst};
use crate::semirings::Semiring;
use crate::{Tr, Label, StateId, EPS_LABEL, NO_LABEL};

#[derive(Debug, Clone, PartialEq)]
pub struct SortedMatcher<F: ExpandedFst> {
    fst: Rc<F>,
    match_type: MatchType,
}

impl<W: Semiring + 'static, F: ExpandedFst<W = W>> Matcher<W> for SortedMatcher<F> {
    type F = F;
    type Iter = IteratorSortedMatcher<W>;

    fn new(fst: Rc<F>, match_type: MatchType) -> Result<Self> {
        Ok(Self { fst, match_type })
    }

    fn iter(&self, state: usize, label: usize) -> Result<Self::Iter> {
        Ok(IteratorSortedMatcher::new(
            self.fst
                .arcs_iter(state)?
                .map(|a| a as *const Tr<W>)
                .collect(),
            label,
            self.match_type,
        ))
    }

    fn final_weight(&self, state: usize) -> Result<Option<*const W>> {
        let final_weight = self.fst.final_weight(state)?;
        let final_weight = final_weight.map(|e| e as *const W);
        Ok(final_weight)
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

    fn fst(&self) -> Rc<Self::F> {
        Rc::clone(&self.fst)
    }
}

#[derive(Clone)]
pub struct IteratorSortedMatcher<W: Semiring> {
    arcs: Vec<*const Tr<W>>,
    match_label: Label,
    pos: usize,
    current_loop: bool,
    match_type: MatchType,
}

impl<W: Semiring> IteratorSortedMatcher<W> {
    pub fn new(arcs: Vec<*const Tr<W>>, match_label: Label, match_type: MatchType) -> Self {
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
                MatchType::MatchInput => {
                    arcs.lower_bound_by(|x| (unsafe { &**x }).ilabel.cmp(&match_label))
                }
                MatchType::MatchOutput => {
                    arcs.lower_bound_by(|x| (unsafe { &**x }).olabel.cmp(&match_label))
                }
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

    fn get_label(&self, arc: &Tr<W>) -> Label {
        match self.match_type {
            MatchType::MatchInput => arc.ilabel,
            MatchType::MatchOutput => arc.olabel,
            _ => panic!("Shouldn't happen : {:?}", self.match_type),
        }
    }
}

impl<W: Semiring> Iterator for IteratorSortedMatcher<W> {
    type Item = IterItemMatcher<W>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_loop {
            self.current_loop = false;
            return Some(IterItemMatcher::EpsLoop);
        }
        if let Some(arc) = self.arcs.get(self.pos) {
            let arc = unsafe { &**arc };
            if self.get_label(arc) == self.match_label {
                self.pos += 1;
                Some(IterItemMatcher::Tr(arc))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<F: ExpandedFst> LookaheadMatcher<F::W> for SortedMatcher<F>
where
    F::W: 'static,
{
    type MatcherData = ();

    fn data(&self) -> Option<&Rc<RefCell<Self::MatcherData>>> {
        unreachable!()
    }

    fn new_with_data(
        _fst: Rc<Self::F>,
        _match_type: MatchType,
        _data: Option<Rc<RefCell<Self::MatcherData>>>,
    ) -> Result<Self>
    where
        Self: std::marker::Sized,
    {
        unreachable!()
    }

    fn create_data<G: ExpandedFst<W = F::W>>(
        _fst: &G,
        _match_type: MatchType,
    ) -> Result<Option<Rc<RefCell<Self::MatcherData>>>> {
        unreachable!()
    }

    fn init_lookahead_fst<LF: ExpandedFst<W = F::W>>(&mut self, _lfst: &Rc<LF>) -> Result<()> {
        unreachable!()
    }

    fn lookahead_fst<LF: ExpandedFst<W = F::W>>(
        &mut self,
        _matcher_state: usize,
        _lfst: &Rc<LF>,
        _lfst_state: usize,
    ) -> Result<bool> {
        unreachable!()
    }

    fn lookahead_label(&self, _state: usize, _label: usize) -> Result<bool> {
        unreachable!()
    }

    fn lookahead_prefix(&self, _tr: &mut Tr<<F as CoreFst>::W>) -> bool {
        unreachable!()
    }

    fn lookahead_weight(&self) -> &<F as CoreFst>::W {
        unreachable!()
    }

    fn prefix_tr(&self) -> &Tr<<F as CoreFst>::W> {
        unreachable!()
    }

    fn prefix_tr_mut(&mut self) -> &mut Tr<<F as CoreFst>::W> {
        unreachable!()
    }

    fn lookahead_weight_mut(&mut self) -> &mut <F as CoreFst>::W {
        unreachable!()
    }
}
