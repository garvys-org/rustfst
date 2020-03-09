use failure::Fallible;
use superslice::Ext;

use crate::algorithms::matchers::{IterItemMatcher, MatchType, Matcher, MatcherFlags};
use crate::fst_traits::{CoreFst, Fst};
use crate::semirings::Semiring;
use crate::{Arc, Label, EPS_LABEL};

#[derive(Debug)]
pub struct SortedMatcher<'fst, F: Fst> {
    fst: &'fst F,
    match_type: MatchType,
}

impl<'fst, F: Fst> Matcher<'fst, F> for SortedMatcher<'fst, F> {
    type Iter = IteratorSortedMatcher<'fst, F::W>;

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

    fn final_weight(&self, state: usize) -> Fallible<Option<&'fst <F as CoreFst>::W>> {
        self.fst.final_weight(state)
    }

    fn match_type(&self) -> MatchType {
        unimplemented!()
    }

    fn flags(&self) -> MatcherFlags {
        MatcherFlags::empty()
    }
}

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

// #[cfg(test)]
// mod tests {
//     use crate::fst_impls::VectorFst;
//     use crate::fst_traits::MutableFst;
//     use crate::semirings::TropicalWeight;
//
//     use super::*;
//
//     #[test]
//     fn lol() -> Fallible<()> {
//         let mut fst = VectorFst::<TropicalWeight>::new();
//         fst.add_states(2);
//         fst.set_start(0)?;
//         fst.set_final(1, TropicalWeight::one())?;
//         fst.emplace_arc(0, 1, 2, 1.2, 1)?;
//         fst.emplace_arc(0, 2, 3, 1.2, 1)?;
//         fst.emplace_arc(0, 3, 4, 1.2, 1)?;
//         fst.emplace_arc(0, 4, 5, 1.2, 1)?;
//
//         let mut matcher = SortedMatcher::new(&fst, MatchType::MatchInput);
//
//         for arc in matcher.iter(0, 2)? {
//             println!("{:?}", arc);
//         }
//
//         Ok(())
//     }
// }
