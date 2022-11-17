use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;
use bitflags::_core::borrow::Borrow;

use crate::{EPS_LABEL, Label, NO_LABEL, Semiring, StateId, Tr};
use crate::algorithms::compose::matchers::{
    IterItemMatcher, Matcher, MatcherFlags, MatcherRewriteMode, MatchType, REQUIRE_PRIORITY,
};
use crate::fst_properties::FstProperties;
use crate::fst_traits::Fst;

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
            self.rewrite_both
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
    matcher_iterator: M::Iter,
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

        let sigma_match = if matcher.iter(state, match_label)?.next().is_some() {
            Some(NO_LABEL)
        } else if has_sigma
            && match_label != EPS_LABEL
            && match_label != NO_LABEL
            && matcher.iter(state, sigma_label)?.next().is_some()
        {
            Some(match_label)
        } else {
            find_empty = true;
            None
        };

        let matcher_iterator = matcher.iter(state, match_label)?;
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
            self.matcher_iterator = self.matcher.iter(self.state, self.sigma_label).unwrap();
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
            let mut sigma_arc : Tr<_> = v.into_tr(self.state, self.match_type).unwrap();

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
