use failure::Fallible;
use superslice::Ext;

use crate::algorithms::matchers::{MatchType, Matcher};
use crate::fst_traits::{CoreFst, Fst};
use crate::semirings::Semiring;
use crate::{Arc, Label, EPS_LABEL, NO_LABEL, NO_STATE_ID};

pub struct SortedMatcher<'fst, F: Fst> {
    fst: &'fst F,
    match_type: MatchType,
    eps_loop: Arc<F::W>,
}

impl<'iter, 'fst: 'iter, F: Fst> Matcher<'iter, 'fst, F> for SortedMatcher<'fst, F> {
    type W = F::W;
    type Iter = IteratorSortedMatcher<'iter, 'fst, Self::W>;

    fn new(fst: &'fst F, match_type: MatchType) -> Fallible<Self> {
        Ok(Self {
            fst,
            match_type,
            eps_loop: match match_type {
                MatchType::MatchInput => Arc::new(NO_LABEL, EPS_LABEL, F::W::one(), NO_STATE_ID),
                MatchType::MatchOutput => Arc::new(EPS_LABEL, NO_LABEL, F::W::one(), NO_STATE_ID),
                _ => bail!("Unsupported match_type : {:?}", match_type),
            },
        })
    }

    fn iter(&'iter mut self, state: usize, label: usize) -> Fallible<Self::Iter> {
        self.eps_loop.nextstate = state;
        Ok(IteratorSortedMatcher::new(
            self.fst.arcs_iter(state)?.collect(),
            label,
            &self.eps_loop,
            self.match_type,
        ))
    }

    fn final_weight(&self, state: usize) -> Fallible<Option<&'iter <F as CoreFst>::W>> {
        self.fst.final_weight(state)
    }
}

pub struct IteratorSortedMatcher<'eps, 'fst, W: Semiring> {
    arcs: Vec<&'fst Arc<W>>,
    match_label: Label,
    pos: usize,
    current_loop: bool,
    eps_loop: &'eps Arc<W>,
    match_type: MatchType,
}

impl<'eps, 'fst: 'eps, W: Semiring> IteratorSortedMatcher<'eps, 'fst, W> {
    pub fn new(
        arcs: Vec<&'fst Arc<W>>,
        match_label: Label,
        eps_loop: &'eps Arc<W>,
        match_type: MatchType,
    ) -> Self {
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
            eps_loop,
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

impl<'eps, 'fst: 'eps, W: Semiring> Iterator for IteratorSortedMatcher<'eps, 'fst, W> {
    type Item = &'eps Arc<W>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_loop {
            self.current_loop = false;
            return Some(&self.eps_loop);
        }
        if let Some(arc) = self.arcs.get(self.pos) {
            if self.get_label(arc) == self.match_label {
                self.pos += 1;
                Some(arc)
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
