use std::fmt::Debug;
use std::iter::Peekable;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;
use bitflags::_core::borrow::Borrow;

use crate::algorithms::compose::matchers::{
    IterItemMatcher, MatchType, Matcher, MatcherFlags, MatcherRewriteMode, REQUIRE_PRIORITY,
};
use crate::fst_properties::FstProperties;
use crate::fst_traits::Fst;
use crate::{Label, Semiring, StateId, Tr, EPS_LABEL, NO_LABEL};

#[derive(Debug, Clone, PartialEq)]
pub struct SigmaMatcher<W, F, B, M>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F>,
    M: Matcher<W, F, B>,
{
    fst: B,
    match_type: MatchType,
    w: PhantomData<(W, F)>,
    sigma_label: Label,
    matcher: Arc<M>,
    rewrite_both: bool,
}

fn has_sigma<W, F, B, M>(state: StateId, matcher: &Arc<M>, sigma_label: Label) -> Result<bool>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F>,
    M: Matcher<W, F, B>,
{
    if sigma_label != NO_LABEL {
        Ok(matcher.iter(state, sigma_label)?.next().is_some())
    } else {
        Ok(false)
    }
}

impl<W, F, B, M> SigmaMatcher<W, F, B, M>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F>,
    M: Matcher<W, F, B>,
{
    pub fn new(
        fst: B,
        match_type: MatchType,
        sigma_label: Label,
        rewrite_mode: MatcherRewriteMode,
        matcher: Arc<M>,
    ) -> Result<Self> {
        if match_type == MatchType::MatchBoth {
            bail!("SigmaMatcher: Bad match type")
        }
        if sigma_label == EPS_LABEL {
            bail!("SigmaMatcher: {} cannot be used as sigma_label", EPS_LABEL)
        }
        let rewrite_both = match rewrite_mode {
            MatcherRewriteMode::MatcherRewriteAuto => {
                fst.borrow().properties().contains(FstProperties::ACCEPTOR)
            }
            MatcherRewriteMode::MatcherRewriteAlways => true,
            MatcherRewriteMode::MatcherRewriteNever => false,
        };
        Ok(Self {
            fst,
            match_type,
            rewrite_both,
            sigma_label,
            matcher,
            w: PhantomData,
        })
    }
    pub fn sigma_label(&self) -> Label {
        self.sigma_label
    }
}

impl<W, F, B, M> Matcher<W, F, B> for SigmaMatcher<W, F, B, M>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F> + Debug,
    M: Matcher<W, F, B>,
{
    type Iter = IteratorSigmaMatcher<W, F, B, M>;

    fn new(_fst: B, _match_type: MatchType) -> Result<Self>
    where
        Self: Sized,
    {
        bail!("This constructor can't be used for initializing SigmaMatcher.")
    }

    fn iter(&self, state: StateId, label: Label) -> Result<Self::Iter> {
        IteratorSigmaMatcher::new(
            state,
            label,
            self.sigma_label,
            self.match_type,
            Arc::clone(&self.matcher),
            self.rewrite_both,
        )
    }

    fn final_weight(&self, state: StateId) -> Result<Option<W>> {
        self.matcher.final_weight(state)
    }

    fn match_type(&self, test: bool) -> Result<MatchType> {
        self.matcher.match_type(test)
    }

    fn flags(&self) -> MatcherFlags {
        if self.sigma_label == NO_LABEL || self.match_type == MatchType::MatchNone {
            self.matcher.flags()
        } else {
            self.matcher.flags() | MatcherFlags::REQUIRE_MATCH
        }
    }

    fn priority(&self, state: StateId) -> Result<usize> {
        if self.sigma_label != NO_LABEL {
            if has_sigma(state, &self.matcher, self.sigma_label)? {
                Ok(REQUIRE_PRIORITY)
            } else {
                self.matcher.priority(state)
            }
        } else {
            self.matcher.priority(state)
        }
    }

    fn fst(&self) -> &B {
        self.matcher.fst()
    }
}

pub struct IteratorSigmaMatcher<W, F, B, M>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F> + Debug,
    M: Matcher<W, F, B>,
{
    state: StateId,
    match_label: Label,
    sigma_label: Label,
    match_type: MatchType,
    matcher: Arc<M>,
    /// Iterator should be done when set to True
    find_empty: bool,
    sigma_match: Option<Label>,
    matcher_iterator: Peekable<M::Iter>,
    has_sigma: bool,
    rewrite_both: bool,
    w: PhantomData<(W, F, B)>,
}

impl<W, F, B, M> IteratorSigmaMatcher<W, F, B, M>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F> + Debug,
    M: Matcher<W, F, B>,
{
    pub fn new(
        state: StateId,
        match_label: Label,
        sigma_label: Label,
        match_type: MatchType,
        matcher: Arc<M>,
        rewrite_both: bool,
    ) -> Result<Self> {
        if match_label == sigma_label && sigma_label != NO_LABEL {
            bail!("SigmaMatcher::Find: bad label (sigma)")
        }

        let mut find_empty = false;
        let has_sigma = has_sigma(state, &matcher, sigma_label)?;

        let mut matcher_itetor_match_label = matcher.iter(state, match_label)?.peekable();
        let (sigma_match, matcher_iterator) = if matcher_itetor_match_label.peek().is_some() {
            (Some(NO_LABEL), matcher_itetor_match_label)
        } else {
            let mut matcher_itetor_sigma_label = matcher.iter(state, sigma_label)?.peekable();
            if has_sigma
                && match_label != EPS_LABEL
                && match_label != NO_LABEL
                && matcher_itetor_sigma_label.peek().is_some()
            {
                (Some(match_label), matcher_itetor_sigma_label)
            } else {
                find_empty = true;

                // The iterator here should be empty. Trick to avoid adding an Option.
                (None, matcher_itetor_sigma_label)
            }
        };

        Ok(Self {
            state,
            match_label,
            sigma_label,
            match_type,
            matcher,
            find_empty,
            sigma_match,
            matcher_iterator,
            has_sigma,
            rewrite_both,
            w: PhantomData,
        })
    }
}

impl<W, F, B, M> Iterator for IteratorSigmaMatcher<W, F, B, M>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F> + Debug,
    M: Matcher<W, F, B>,
{
    type Item = IterItemMatcher<W>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.find_empty {
            return None;
        }

        let v = if let Some(v_iterator) = self.matcher_iterator.next() {
            v_iterator
        } else if self.has_sigma && self.sigma_match.unwrap() == NO_LABEL && self.match_label > 0 {
            // TODO : Move to FallibleIterator
            self.matcher_iterator = self
                .matcher
                .iter(self.state, self.sigma_label)
                .unwrap()
                .peekable();
            self.sigma_match = Some(self.match_label);
            if let Some(v_iterator) = self.matcher_iterator.next() {
                v_iterator
            } else {
                return None;
            }
        } else {
            return None;
        };

        if self.sigma_match.unwrap() == NO_LABEL {
            Some(v)
        } else {
            let mut sigma_arc: Tr<_> = v.into_tr(self.state, self.match_type).unwrap();

            if self.rewrite_both {
                if sigma_arc.ilabel == self.sigma_label {
                    sigma_arc.ilabel = self.sigma_match.unwrap();
                }
                if sigma_arc.olabel == self.sigma_label {
                    sigma_arc.olabel = self.sigma_match.unwrap();
                }
            } else if self.match_type == MatchType::MatchInput {
                sigma_arc.ilabel = self.sigma_match.unwrap();
            } else {
                sigma_arc.olabel = self.sigma_match.unwrap();
            }
            Some(IterItemMatcher::Tr(sigma_arc))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::algorithms::compose::matchers::SortedMatcher;
    use crate::algorithms::compose::{ComposeFst, ComposeFstOpOptions};
    use crate::fst_traits::MutableFst;
    use crate::prelude::compose::compose_filters::SequenceComposeFilterBuilder;
    use crate::prelude::{tr_sort, ILabelCompare, OLabelCompare, VectorFst};
    use crate::semirings::TropicalWeight;
    use crate::utils::acceptor;
    use crate::SymbolTable;

    use super::*;

    fn create_symt() -> SymbolTable {
        let mut symt = SymbolTable::new();
        symt.add_symbol("<sigma>"); // 1
        symt.add_symbol("play"); // 2
        symt.add_symbol("bowie"); // 3
        symt.add_symbol("queen"); // 4
        symt.add_symbol("please"); // 5

        symt
    }

    fn query_fst(symt: &Arc<SymbolTable>) -> VectorFst<TropicalWeight> {
        let labels = vec![
            symt.get_label("play").unwrap(),
            symt.get_label("bowie").unwrap(),
            symt.get_label("please").unwrap(),
        ];
        let mut f: VectorFst<_> = acceptor(labels.as_slice(), TropicalWeight::one());
        f.set_input_symbols(Arc::clone(symt));
        f.set_output_symbols(Arc::clone(symt));
        f
    }

    fn grammar_fst_loop(symt: &Arc<SymbolTable>) -> VectorFst<TropicalWeight> {
        let mut fst = VectorFst::new();
        fst.add_states(4);

        let label_play = symt.get_label("play").unwrap();
        let label_bowie = symt.get_label("bowie").unwrap();
        let label_queen = symt.get_label("queen").unwrap();
        let label_please = symt.get_label("please").unwrap();

        fst.emplace_tr(0, label_play, label_play, TropicalWeight::one(), 1)
            .unwrap();
        fst.emplace_tr(1, label_bowie, label_bowie, TropicalWeight::one(), 2)
            .unwrap();
        fst.emplace_tr(1, label_queen, label_queen, TropicalWeight::one(), 2)
            .unwrap();
        fst.emplace_tr(2, label_please, label_please, TropicalWeight::one(), 3)
            .unwrap();

        fst.set_start(0).unwrap();
        fst.set_final(3, TropicalWeight::one()).unwrap();

        fst.set_input_symbols(Arc::clone(symt));
        fst.set_output_symbols(Arc::clone(symt));

        fst
    }

    fn xp_loop(
        symt: &Arc<SymbolTable>,
        q_fst: VectorFst<TropicalWeight>,
    ) -> VectorFst<TropicalWeight> {
        let mut g_fst = grammar_fst_loop(symt);
        tr_sort(&mut g_fst, ILabelCompare {});

        let compose_lazy = ComposeFst::<
            _,                                                 // W
            _,                                                 // F1
            _,                                                 // F2
            _,                                                 // B1
            _,                                                 // B2
            SortedMatcher<_, _, _>,                            // M1
            SortedMatcher<_, _, _>,                            // M2
            SequenceComposeFilterBuilder<_, _, _, _, _, _, _>, // CFB
        >::new(q_fst, g_fst)
        .unwrap();
        let mut compose_vec: VectorFst<_> = compose_lazy.compute().unwrap();

        compose_vec.set_input_symbols(Arc::clone(symt));
        compose_vec.set_output_symbols(Arc::clone(symt));

        compose_vec
    }

    fn grammar_fst_sigma(symt: &Arc<SymbolTable>) -> VectorFst<TropicalWeight> {
        let mut fst = VectorFst::new();
        fst.add_states(4);

        let label_play = symt.get_label("play").unwrap();
        let label_sigma = symt.get_label("<sigma>").unwrap();
        let label_please = symt.get_label("please").unwrap();

        fst.emplace_tr(0, label_play, label_play, TropicalWeight::one(), 1)
            .unwrap();
        fst.emplace_tr(1, label_sigma, label_sigma, TropicalWeight::one(), 2)
            .unwrap();
        fst.emplace_tr(2, label_please, label_please, TropicalWeight::one(), 3)
            .unwrap();

        fst.set_start(0).unwrap();
        fst.set_final(3, TropicalWeight::one()).unwrap();

        fst.set_input_symbols(Arc::clone(symt));
        fst.set_output_symbols(Arc::clone(symt));

        fst
    }

    fn xp_sigma(
        symt: &Arc<SymbolTable>,
        q_fst: VectorFst<TropicalWeight>,
    ) -> VectorFst<TropicalWeight> {
        let mut g_fst = grammar_fst_sigma(symt);
        tr_sort(&mut g_fst, ILabelCompare {});

        let compose_fst_op_opts = ComposeFstOpOptions::new(
            None,
            SigmaMatcher::new(
                g_fst.clone(),
                MatchType::MatchInput,
                symt.get_label("<sigma>").unwrap(),
                MatcherRewriteMode::MatcherRewriteAuto,
                Arc::new(SortedMatcher::new(g_fst.clone(), MatchType::MatchInput).unwrap()),
            )
            .unwrap(),
            None,
            None,
        );
        let compose_lazy = ComposeFst::<
            _,                                                 // W
            _,                                                 // F1
            _,                                                 // F2
            _,                                                 // B1
            _,                                                 // B2
            SortedMatcher<_, _, _>,                            // M1
            SigmaMatcher<_, _, _, SortedMatcher<_, _, _>>,     // M2
            SequenceComposeFilterBuilder<_, _, _, _, _, _, _>, // CFB
        >::new_with_options(q_fst, g_fst, compose_fst_op_opts)
        .unwrap();
        let mut compose_vec: VectorFst<_> = compose_lazy.compute().unwrap();

        compose_vec.set_input_symbols(Arc::clone(symt));
        compose_vec.set_output_symbols(Arc::clone(symt));

        compose_vec
    }

    #[test]
    fn test_sigma_matcher() -> Result<()> {
        let symt = Arc::new(create_symt());

        let mut q_fst = query_fst(&symt);
        tr_sort(&mut q_fst, OLabelCompare {});

        let composed_fst_loop = xp_loop(&symt, q_fst.clone());
        let composed_fst_sigma = xp_sigma(&symt, q_fst);

        assert_eq!(composed_fst_loop, composed_fst_sigma);

        Ok(())
    }
}
