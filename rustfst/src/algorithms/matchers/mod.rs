use std::slice::Iter as IterSlice;

use failure::Fallible;
use superslice::Ext;

use crate::{Arc, NO_LABEL, NO_STATE_ID};
use crate::{EPS_LABEL, Label, StateId};
use crate::fst_traits::ExpandedFst;
use crate::semirings::Semiring;

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

/// Matchers find and iterate through requested labels at FST states. In the
/// simplest form, these are just some associative map or search keyed on labels.
/// More generally, they may implement matching special labels that represent
/// sets of labels such as sigma (all), rho (rest), or phi (fail).
pub trait Matcher<'a> {
    type W: Semiring + 'a;
    type Iter: Iterator<Item = &'a Arc<Self::W>>;

    fn iter(&mut self, state: StateId, label: Label) -> Fallible<Self::Iter>;
}

struct SortedMatcher<'a, F: ExpandedFst>  {
    fst: &'a F,
    match_type: MatchType,
    eps_loop: Arc<F::W>
}

impl<'a, F: ExpandedFst> SortedMatcher<'a, F> {
    pub fn new(fst: &'a F, match_type: MatchType) -> Self {
        // TODO: Add check label sorted
        Self { fst, match_type, eps_loop: Arc::new(NO_LABEL, EPS_LABEL, F::W::one(), NO_STATE_ID) }
    }
}

impl<'a, F: ExpandedFst> Matcher<'a> for SortedMatcher<'a, F> {
    type W = F::W;
    type Iter = IteratorSortedMatcher<'a, F::W>;

    fn iter(&mut self, state: usize, label: usize) -> Fallible<Self::Iter> {
        Ok(IteratorSortedMatcher::new(
            self.fst.arcs_iter(state)?.collect(),
            label,
            &self.eps_loop
        ))
    }
}

struct IteratorSortedMatcher<'a, W: Semiring>{
    arcs: Vec<&'a Arc<W>>,
    match_label: Label,
    pos: usize,
    current_loop: bool,
    eps_loop: &'a Arc<W>,
}

impl<'a, W: Semiring> IteratorSortedMatcher<'a, W> {
    pub fn new(arcs: Vec<&'a Arc<W>>, match_label: Label, eps_loop: &'a Arc<W>) -> Self {

        // If we have to match epsilon, an epsilon loop is added
        let current_loop = match_label == EPS_LABEL;

        // When matching epsilon, the first arc is supposed to be labeled as such
        let pos = if current_loop {
            0
        } else {
            arcs.lower_bound_by(|x| x.ilabel.cmp(&match_label))
        };

        Self {
            arcs,
            match_label,
            pos,
            current_loop,
            eps_loop,
        }
    }
}

impl<'a, W: Semiring> Iterator for IteratorSortedMatcher<'a, W> {
    type Item = &'a Arc<W>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_loop {
            self.current_loop = false;
            unimplemented!()
//            return Some(&self.eps_loop)
        }
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
    use crate::fst_impls::VectorFst;
    use crate::fst_traits::MutableFst;
    use crate::semirings::TropicalWeight;

    use super::*;

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
