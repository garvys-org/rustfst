use crate::fst_traits::ExpandedFst;
use failure::Fallible;
use std::slice::Iter as IterSlice;

#[derive(Copy, Debug, PartialOrd, PartialEq, Clone)]
/// Specifies matcher action
enum MatchType {
    /// Match input label
    MatchInput,
    /// Match output label
    MatchOutput,
    /// Match input or output label
    MatchBoth,
    /// Match anything
    MatchNone,
    /// Otherwise, match unknown
    MatchUnknown,
}
use crate::semirings::Semiring;
use crate::{Arc, Label, StateId};

/// Matchers find and iterate through requested labels at FST states. In the
/// simplest form, these are just some associative map or search keyed on labels.
/// More generally, they may implement matching special labels that represent
/// sets of labels such as sigma (all), rho (rest), or phi (fail).
pub trait Matcher<'a> {
    type W: Semiring + 'a;
    type Iter: Iterator<Item = &'a Arc<Self::W>>;

    fn iter(&mut self, state: StateId, label: Label) -> Fallible<Self::Iter>;
}

struct SortedMatcher<'a, F: ExpandedFst> {
    fst: &'a F,
    match_type: MatchType,
}

impl<'a, F: ExpandedFst> SortedMatcher<'a, F> {
    pub fn new(fst: &'a F, match_type: MatchType) -> Self {
        Self { fst, match_type }
    }
}

impl<'a, F: ExpandedFst> Matcher<'a> for SortedMatcher<'a, F> {
    type W = F::W;
    type Iter = IteratorSortedMatcher<'a, F::W>;

    fn iter(&mut self, state: usize, label: usize) -> Fallible<Self::Iter> {
        Ok(IteratorSortedMatcher::new(
            self.fst.arcs_iter(state)?.collect(),
            label,
        ))
    }
}

struct IteratorSortedMatcher<'a, W: Semiring> {
    arcs: Vec<&'a Arc<W>>,
    match_label: Label,
    pos: usize,
}

use superslice::Ext;
impl<'a, W: Semiring> IteratorSortedMatcher<'a, W> {
    pub fn new(arcs: Vec<&'a Arc<W>>, match_label: Label) -> Self {
        let pos = arcs.lower_bound_by(|x| x.ilabel.cmp(&match_label));
        Self {
            arcs,
            match_label,
            pos,
        }
    }
}

impl<'a, W: Semiring> Iterator for IteratorSortedMatcher<'a, W> {
    type Item = &'a Arc<W>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(arc) = self.arcs.get(self.pos) {
            if arc.ilabel == self.match_label {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fst_impls::VectorFst;
    use crate::fst_traits::MutableFst;
    use crate::semirings::TropicalWeight;

    #[test]
    fn lol() -> Fallible<()> {
        let mut fst = VectorFst::<TropicalWeight>::new();
        fst.add_states(2);
        fst.set_start(0)?;
        fst.set_final(1, TropicalWeight::one())?;
        fst.emplace_arc(0, 1, 2, 1.2, 1)?;
        fst.emplace_arc(0, 2, 3, 1.2, 1)?;
        fst.emplace_arc(0, 3, 4, 1.2, 1)?;
        fst.emplace_arc(0, 4, 5, 1.2, 1)?;

        let mut matcher = SortedMatcher {
            fst: &fst,
            match_type: MatchType::MatchInput,
        };

        for arc in matcher.iter(0, 2)? {
            println!("{:?}", arc);
        }

        Ok(())
    }
}
