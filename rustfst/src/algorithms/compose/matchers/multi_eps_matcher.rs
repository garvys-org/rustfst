use std::borrow::Borrow;
use std::fmt::Debug;
use std::iter::Peekable;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;
use itertools::Itertools;
use nom::lib::std::collections::BTreeSet;

use bitflags::bitflags;

use crate::algorithms::compose::matchers::{IterItemMatcher, MatchType, Matcher, MatcherFlags};
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{Label, StateId, EPS_LABEL, NO_LABEL};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct MultiEpsMatcherFlags: u32 {
        const MULTI_EPS_LOOP =  1u32;
        const MULTI_EPS_LIST =  2u32;
    }
}

#[derive(Clone, Debug)]
pub struct MultiEpsMatcher<W, F, B, M>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F> + Debug,
    M: Matcher<W, F, B>,
{
    matcher: Arc<M>,
    flags: MultiEpsMatcherFlags,
    multi_eps_labels: CompactSet<Label>,
    ghost: PhantomData<(W, F, B)>,
}

pub struct IteratorMultiEpsMatcher<W, F, B, M>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F> + Debug,
    M: Matcher<W, F, B>,
{
    iter_matcher: Option<Peekable<M::Iter>>,
    iter_labels: Option<(Vec<Label>, usize)>,
    matcher: Arc<M>,
    matcher_state: StateId,
    done: bool,
    ghost: PhantomData<W>,
}

impl<W, F, B, M> Clone for IteratorMultiEpsMatcher<W, F, B, M>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F> + Debug,
    M: Matcher<W, F, B>,
{
    fn clone(&self) -> Self {
        unimplemented!()
        // Self {
        //     iter_matcher: self.iter_matcher.clone(),
        //     iter_labels: self.iter_labels.clone(),
        //     matcher: Arc::clone(&self.matcher),
        //     ghost: PhantomData,
        //     done: self.done,
        //     matcher_state: self.matcher_state,
        // }
    }
}

impl<W, F, B, M> Iterator for IteratorMultiEpsMatcher<W, F, B, M>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F> + Debug,
    M: Matcher<W, F, B>,
{
    type Item = IterItemMatcher<W>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref mut matcher_iter) = &mut self.iter_matcher {
            let res = matcher_iter.next();
            let done = res.is_none();
            if done {
                if let Some((multi_eps_labels, pos_labels)) = &mut self.iter_labels {
                    if *pos_labels >= multi_eps_labels.len() {
                        return None;
                    }
                    *pos_labels += 1;
                    while *pos_labels < multi_eps_labels.len() {
                        let mut it = self
                            .matcher
                            .iter(self.matcher_state, multi_eps_labels[*pos_labels] as Label)
                            .unwrap()
                            .peekable();
                        if it.peek().is_some() {
                            *matcher_iter = it;
                            break;
                        }
                        *pos_labels += 1;
                    }
                    if *pos_labels < multi_eps_labels.len() {
                        self.done = false;
                        res
                    } else {
                        *matcher_iter = self
                            .matcher
                            .iter(self.matcher_state, NO_LABEL)
                            .unwrap()
                            .peekable();
                        matcher_iter.next()
                    }
                } else {
                    res
                }
            } else {
                res
            }
        } else if self.done {
            None
        } else {
            self.done = true;
            Some(IterItemMatcher::EpsLoop)
        }
    }
}

impl<W, F, B, M> MultiEpsMatcher<W, F, B, M>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F> + Debug,
    M: Matcher<W, F, B>,
{
    pub fn new_with_opts<IM: Into<Option<Arc<M>>>>(
        fst: B,
        match_type: MatchType,
        flags: MultiEpsMatcherFlags,
        matcher: IM,
    ) -> Result<Self> {
        let matcher = matcher
            .into()
            .unwrap_or_else(|| Arc::new(M::new(fst, match_type).unwrap()));
        Ok(Self {
            matcher,
            flags,
            multi_eps_labels: CompactSet::new(NO_LABEL),
            ghost: PhantomData,
        })
    }

    pub fn matcher(&self) -> &Arc<M> {
        &self.matcher
    }

    pub fn clear_multi_eps_labels(&mut self) {
        self.multi_eps_labels.clear()
    }

    pub fn add_multi_eps_label(&mut self, label: Label) -> Result<()> {
        if label == EPS_LABEL {
            bail!("MultiEpsMatcher: Bad multi-eps label: 0")
        }
        self.multi_eps_labels.insert(label);
        Ok(())
    }

    pub fn remove_multi_eps_label(&mut self, label: Label) -> Result<()> {
        if label == EPS_LABEL {
            bail!("MultiEpsMatcher: Bad multi-eps label: 0")
        }
        self.multi_eps_labels.erase(label);
        Ok(())
    }
}

impl<W, F, B, M> Matcher<W, F, B> for MultiEpsMatcher<W, F, B, M>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F> + Debug,
    M: Matcher<W, F, B>,
{
    type Iter = IteratorMultiEpsMatcher<W, F, B, M>;

    fn new(fst: B, match_type: MatchType) -> Result<Self> {
        Self::new_with_opts(
            fst,
            match_type,
            MultiEpsMatcherFlags::MULTI_EPS_LOOP | MultiEpsMatcherFlags::MULTI_EPS_LIST,
            None,
        )
    }

    fn iter(&self, state: StateId, label: Label) -> Result<Self::Iter> {
        let (iter_matcher, iter_labels) = if label == EPS_LABEL {
            (Some(self.matcher.iter(state, EPS_LABEL)?.peekable()), None)
        } else if label == NO_LABEL {
            if self.flags.contains(MultiEpsMatcherFlags::MULTI_EPS_LIST) {
                // TODO: Didn't find a way to store the iterator in IteratorMultiEpsMatcher.
                let multi_eps_labels = self.multi_eps_labels.iter().cloned().collect_vec();

                let mut iter_matcher = None;
                let mut pos_labels = 0;
                while pos_labels < multi_eps_labels.len() {
                    let mut it = self
                        .matcher
                        .iter(state, multi_eps_labels[pos_labels])?
                        .peekable();
                    if it.peek().is_some() {
                        iter_matcher = Some(it);
                        break;
                    }
                    pos_labels += 1;
                }

                if pos_labels < multi_eps_labels.len() {
                    (iter_matcher, Some((multi_eps_labels, pos_labels)))
                } else {
                    (Some(self.matcher.iter(state, NO_LABEL)?.peekable()), None)
                }
            } else {
                (Some(self.matcher.iter(state, NO_LABEL)?.peekable()), None)
            }
        } else if self.flags.contains(MultiEpsMatcherFlags::MULTI_EPS_LOOP)
            && self.multi_eps_labels.contains(&label)
        {
            // Empty iter
            (None, None)
        } else {
            (Some(self.matcher.iter(state, label)?.peekable()), None)
        };
        Ok(IteratorMultiEpsMatcher {
            iter_matcher,
            iter_labels,
            matcher: Arc::clone(&self.matcher),
            ghost: PhantomData,
            done: false,
            matcher_state: state,
        })
    }

    fn final_weight(&self, state: StateId) -> Result<Option<W>> {
        self.matcher.final_weight(state)
    }

    fn match_type(&self, test: bool) -> Result<MatchType> {
        self.matcher.match_type(test)
    }

    fn flags(&self) -> MatcherFlags {
        self.matcher.flags()
    }

    fn priority(&self, state: StateId) -> Result<usize> {
        self.matcher.priority(state)
    }

    fn fst(&self) -> &B {
        self.matcher.fst()
    }
}

trait CompactSetKey: Copy + Ord {
    fn add(self, v: usize) -> Self;
    fn sub(self, v: usize) -> Self;
}

impl CompactSetKey for usize {
    fn add(self, v: usize) -> Self {
        self + v
    }
    fn sub(self, v: usize) -> Self {
        self - v
    }
}

impl CompactSetKey for u32 {
    fn add(self, v: usize) -> Self {
        self + v as u32
    }
    fn sub(self, v: usize) -> Self {
        self - v as u32
    }
}

#[derive(Clone, Debug)]
struct CompactSet<K> {
    set: BTreeSet<K>,
    min_key: K,
    max_key: K,
    no_key: K,
}

impl<K: Copy + Ord> CompactSet<K> {
    pub fn new(no_key: K) -> Self {
        Self {
            set: BTreeSet::new(),
            min_key: no_key,
            max_key: no_key,
            no_key,
        }
    }

    pub fn insert(&mut self, key: K) {
        self.set.insert(key);
        if self.min_key == self.no_key || key < self.min_key {
            self.min_key = key;
        }
        if self.max_key == self.no_key || self.max_key > key {
            self.max_key = key;
        }
    }

    pub fn clear(&mut self) {
        self.set.clear();
        self.min_key = self.no_key;
        self.max_key = self.no_key;
    }

    pub fn iter(&self) -> std::collections::btree_set::Iter<K> {
        self.set.iter()
    }

    #[allow(unused)]
    pub fn lower_bound(&self) -> K {
        self.min_key
    }

    #[allow(unused)]
    pub fn upper_bound(&self) -> K {
        self.max_key
    }
}

impl<K: CompactSetKey> CompactSet<K> {
    pub fn erase(&mut self, key: K) {
        self.set.remove(&key);
        if self.set.is_empty() {
            self.min_key = self.no_key;
            self.max_key = self.no_key;
        } else if key == self.min_key {
            self.min_key = self.min_key.add(1);
        } else if key == self.max_key {
            self.max_key = self.max_key.sub(1);
        }
    }

    pub fn contains(&self, key: &K) -> bool {
        if self.min_key == self.no_key || *key < self.min_key || *key > self.max_key {
            // out of range
            false
        } else if self.min_key != self.no_key
            && self.max_key.add(1) as K == self.min_key.add(self.set.len())
        {
            // dense range
            true
        } else {
            self.set.contains(key)
        }
    }
}
