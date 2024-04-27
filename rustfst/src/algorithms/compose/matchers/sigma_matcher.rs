use std::borrow::Borrow;
use std::fmt::Debug;
use std::iter::Peekable;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::compose::matchers::{
    IterItemMatcher, MatchType, Matcher, MatcherFlags, MatcherRewriteMode, REQUIRE_PRIORITY,
};
use crate::fst_properties::FstProperties;
use crate::fst_traits::Fst;
use crate::{Label, Semiring, StateId, Tr, EPS_LABEL, NO_LABEL};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct SigmaMatcher<W, F, B, M>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F>,
    M: Matcher<W, F, B>,
{
    match_type: MatchType,
    w: PhantomData<(W, F, B)>,
    sigma_label: Label,
    matcher: Arc<M>,
    rewrite_both: bool,
    sigma_allowed_matches: Option<HashSet<Label>>,
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
        match_type: MatchType,
        sigma_label: Label,
        rewrite_mode: MatcherRewriteMode,
        matcher: Arc<M>,
        sigma_allowed_matches: Option<HashSet<Label>>,
    ) -> Result<Self> {
        if match_type == MatchType::MatchBoth {
            bail!("SigmaMatcher: Bad match type")
        }
        if sigma_label == EPS_LABEL {
            bail!("SigmaMatcher: {} cannot be used as sigma_label", EPS_LABEL)
        }
        let rewrite_both = match rewrite_mode {
            MatcherRewriteMode::MatcherRewriteAuto => matcher
                .fst()
                .borrow()
                .properties()
                .contains(FstProperties::ACCEPTOR),
            MatcherRewriteMode::MatcherRewriteAlways => true,
            MatcherRewriteMode::MatcherRewriteNever => false,
        };
        Ok(Self {
            match_type,
            rewrite_both,
            sigma_label,
            matcher,
            w: PhantomData,
            sigma_allowed_matches,
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
            &self.sigma_allowed_matches,
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

fn is_match_label_allowed(
    sigma_allowed_matches: &Option<HashSet<Label>>,
    match_label: Label,
) -> bool {
    if let Some(allowed_matches) = sigma_allowed_matches {
        allowed_matches.contains(&match_label)
    } else {
        true
    }
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
        sigma_allowed_matches: &Option<HashSet<Label>>,
    ) -> Result<Self> {
        if match_label == sigma_label && sigma_label != NO_LABEL {
            bail!("SigmaMatcher::Find: bad label (sigma)")
        }

        let mut find_empty = false;
        let has_sigma = has_sigma(state, &matcher, sigma_label)?;

        let mut matcher_iterator_match_label = matcher.iter(state, match_label)?.peekable();
        let (sigma_match, matcher_iterator) = if matcher_iterator_match_label.peek().is_some() {
            (Some(NO_LABEL), matcher_iterator_match_label)
        } else {
            let mut matcher_iterator_sigma_label = matcher.iter(state, sigma_label)?.peekable();
            if has_sigma
                && match_label != EPS_LABEL
                && match_label != NO_LABEL
                && is_match_label_allowed(sigma_allowed_matches, match_label)
                && matcher_iterator_sigma_label.peek().is_some()
            {
                (Some(match_label), matcher_iterator_sigma_label)
            } else {
                find_empty = true;

                // The iterator here should be empty. Trick to avoid adding an Option.
                (None, matcher_iterator_sigma_label)
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

    // This assusmes, the iterator is not done
    pub fn value_openfst(&mut self) -> IterItemMatcher<W> {
        if self.sigma_match.unwrap() == NO_LABEL {
            self.matcher_iterator.peek().unwrap().clone()
        } else {
            let mut sigma_arc: Tr<_> = self
                .matcher_iterator
                .peek()
                .unwrap()
                .clone()
                .into_tr(self.state, self.match_type)
                .unwrap();

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
            IterItemMatcher::Tr(sigma_arc)
        }
    }

    pub fn next_openfst(&mut self) {
        let r = self.matcher_iterator.next();
        if r.is_none()
            && self.has_sigma
            && self.sigma_match.unwrap() == NO_LABEL
            && self.match_label > 0
        {
            self.matcher_iterator = self
                .matcher
                .iter(self.state, self.sigma_label)
                .unwrap()
                .peekable();
            self.sigma_match = Some(self.match_label);
        }
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

        self.matcher_iterator.peek()?;

        let v = self.value_openfst();
        self.next_openfst();
        Some(v)
    }
}

#[cfg(test)]
mod tests {
    use crate::algorithms::compose::matchers::SortedMatcher;
    use crate::algorithms::compose::{ComposeFst, ComposeFstOpOptions};
    use crate::fst_traits::{MutableFst, SerializableFst};
    use crate::prelude::compose::compose_filters::SequenceComposeFilterBuilder;
    use crate::prelude::{tr_sort, ILabelCompare, OLabelCompare, VectorFst};
    use crate::semirings::TropicalWeight;
    use crate::utils::acceptor;
    use crate::SymbolTable;
    use path_abs::{PathAbs, PathMut, PathOps};
    use std::path::PathBuf;

    use super::*;

    fn create_symt() -> SymbolTable {
        let mut symt = SymbolTable::new();
        symt.add_symbol("<sigma>"); // 1
        symt.add_symbol("play"); // 2
        symt.add_symbol("bowie"); // 3
        symt.add_symbol("queen"); // 4
        symt.add_symbol("please"); // 5
        symt.add_symbol("radiohead");

        symt
    }

    fn query_fst_bowie(symt: &Arc<SymbolTable>) -> VectorFst<TropicalWeight> {
        query_fst(symt, "bowie")
    }

    fn query_fst_queen(symt: &Arc<SymbolTable>) -> VectorFst<TropicalWeight> {
        query_fst(symt, "queen")
    }
    fn query_fst_radiohead(symt: &Arc<SymbolTable>) -> VectorFst<TropicalWeight> {
        query_fst(symt, "radiohead")
    }

    fn query_fst(symt: &Arc<SymbolTable>, artist_name: &str) -> VectorFst<TropicalWeight> {
        let labels = vec![
            symt.get_label("play").unwrap(),
            symt.get_label(artist_name).unwrap(),
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
        let label_radiohead = symt.get_label("radiohead").unwrap();
        let label_please = symt.get_label("please").unwrap();

        fst.emplace_tr(0, label_play, label_play, TropicalWeight::one(), 1)
            .unwrap();
        fst.emplace_tr(1, label_bowie, label_bowie, TropicalWeight::one(), 2)
            .unwrap();
        fst.emplace_tr(1, label_queen, label_queen, TropicalWeight::one(), 2)
            .unwrap();
        fst.emplace_tr(
            1,
            label_radiohead,
            label_radiohead,
            TropicalWeight::one(),
            2,
        )
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
        sigma_allowed_matches: Option<Vec<String>>,
    ) -> VectorFst<TropicalWeight> {
        let mut g_fst = grammar_fst_sigma(symt);
        tr_sort(&mut g_fst, ILabelCompare {});

        let compose_fst_op_opts = ComposeFstOpOptions::new(
            None,
            SigmaMatcher::new(
                MatchType::MatchInput,
                symt.get_label("<sigma>").unwrap(),
                MatcherRewriteMode::MatcherRewriteAuto,
                Arc::new(SortedMatcher::new(g_fst.clone(), MatchType::MatchInput).unwrap()),
                sigma_allowed_matches
                    .map(|e| e.iter().map(|s| symt.get_label(s).unwrap()).collect()),
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

        let mut q_fst = query_fst_bowie(&symt);
        tr_sort(&mut q_fst, OLabelCompare {});

        let composed_fst_loop = xp_loop(&symt, q_fst.clone());
        let composed_fst_sigma = xp_sigma(&symt, q_fst, None);

        assert_eq!(composed_fst_loop, composed_fst_sigma);

        Ok(())
    }

    #[test]
    fn test_sigma_matcher_with_limited_allowed_values() -> Result<()> {
        let symt = Arc::new(create_symt());
        let allowed_sigma_match = Some(vec!["radiohead".to_string(), "queen".to_string()]);

        // Radiohead should work
        {
            let mut q_fst = query_fst_radiohead(&symt);
            tr_sort(&mut q_fst, OLabelCompare {});

            let composed_fst_loop = xp_loop(&symt, q_fst.clone());
            let composed_fst_sigma = xp_sigma(&symt, q_fst, allowed_sigma_match.clone());

            assert_eq!(
                composed_fst_loop, composed_fst_sigma,
                "Radiohead should match"
            );
        }

        // Queen should work
        {
            let mut q_fst = query_fst_queen(&symt);
            tr_sort(&mut q_fst, OLabelCompare {});

            let composed_fst_loop = xp_loop(&symt, q_fst.clone());
            let composed_fst_sigma = xp_sigma(&symt, q_fst, allowed_sigma_match.clone());

            assert_eq!(composed_fst_loop, composed_fst_sigma, "Queen should match");
        }

        // Bowie should not work
        {
            let mut q_fst = query_fst_bowie(&symt);
            tr_sort(&mut q_fst, OLabelCompare {});

            let composed_fst_loop = xp_loop(&symt, q_fst.clone());
            let composed_fst_sigma = xp_sigma(&symt, q_fst, allowed_sigma_match.clone());

            assert_ne!(
                composed_fst_loop, composed_fst_sigma,
                "Bowie should NOT match"
            );
        }

        Ok(())
    }

    #[test]
    fn test_sigma_matcher_2() -> Result<()> {
        let mut path_folder =
            PathAbs::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap())?;
        path_folder.append("rustfst-tests-data")?;
        path_folder.append("sigma-matcher-2")?;

        let mut left_fst = VectorFst::<TropicalWeight>::read(path_folder.join("left.fst"))?;
        let mut right_fst = VectorFst::<TropicalWeight>::read(path_folder.join("right.fst"))?;
        let symt = Arc::new(SymbolTable::read(path_folder.join("symt.bin"))?);

        left_fst.set_input_symbols(symt.clone());
        left_fst.set_output_symbols(symt.clone());
        right_fst.set_input_symbols(symt.clone());
        right_fst.set_output_symbols(symt.clone());

        tr_sort(&mut left_fst, OLabelCompare {});
        tr_sort(&mut right_fst, ILabelCompare {});

        let compose_fst_op_opts = ComposeFstOpOptions::new(
            None,
            SigmaMatcher::new(
                MatchType::MatchInput,
                symt.get_label("<sigma>").unwrap(),
                MatcherRewriteMode::MatcherRewriteAuto,
                Arc::new(SortedMatcher::new(right_fst.clone(), MatchType::MatchInput).unwrap()),
                None,
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
        >::new_with_options(left_fst, right_fst, compose_fst_op_opts)
        .unwrap();
        let compose_vec: VectorFst<TropicalWeight> = compose_lazy.compute().unwrap();

        assert_eq!(compose_vec.string_paths_iter()?.count(), 4);

        Ok(())
    }
}
