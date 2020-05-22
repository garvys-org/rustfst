use std::iter::Peekable;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, SubAssign};
use std::sync::Arc;

use anyhow::Result;
use itertools::Itertools;
use nom::lib::std::collections::BTreeSet;

use bitflags::bitflags;

use crate::algorithms::compose::matchers::{IterItemMatcher, MatchType, Matcher, MatcherFlags};
use crate::semirings::Semiring;
use crate::{Label, StateId, EPS_LABEL, NO_LABEL};

bitflags! {
    pub struct MultiEpsMatcherFlags: u32 {
        const MULTI_EPS_LOOP =  1u32 << 0;
        const MULTI_EPS_LIST =  1u32 << 1;
    }
}

#[derive(Clone, Debug)]
pub struct MultiEpsMatcher<W, M> {
    matcher: Arc<M>,
    flags: MultiEpsMatcherFlags,
    w: PhantomData<W>,
    multi_eps_labels: CompactSet<Label>,
}

pub struct IteratorMultiEpsMatcher<W: Semiring, M: Matcher<W>> {
    iter_matcher: Option<Peekable<M::Iter>>,
    iter_labels: Option<(Vec<usize>, usize)>,
    matcher: Arc<M>,
    matcher_state: StateId,
    ghost: PhantomData<W>,
    done: bool,
}

impl<W: Semiring, M: Matcher<W>> Clone for IteratorMultiEpsMatcher<W, M> {
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

impl<W: Semiring, M: Matcher<W>> Iterator for IteratorMultiEpsMatcher<W, M> {
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
                            .iter(self.matcher_state, multi_eps_labels[*pos_labels])
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
        } else {
            if self.done {
                None
            } else {
                self.done = true;
                Some(IterItemMatcher::EpsLoop)
            }
        }
    }
}

impl<W: Semiring, M: Matcher<W>> MultiEpsMatcher<W, M> {
    pub fn new_with_opts<IM: Into<Option<Arc<M>>>>(
        fst: Arc<<Self as Matcher<W>>::F>,
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
            w: PhantomData,
            multi_eps_labels: CompactSet::new(NO_LABEL),
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

impl<W: Semiring, M: Matcher<W>> Matcher<W> for MultiEpsMatcher<W, M> {
    type F = M::F;
    type Iter = IteratorMultiEpsMatcher<W, M>;

    fn new(fst: Arc<Self::F>, match_type: MatchType) -> Result<Self> {
        Self::new_with_opts(
            fst,
            match_type,
            MultiEpsMatcherFlags::MULTI_EPS_LOOP | MultiEpsMatcherFlags::MULTI_EPS_LIST,
            None,
        )
    }

    fn iter(&self, state: usize, label: usize) -> Result<Self::Iter> {
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

    fn final_weight(&self, state: usize) -> Result<Option<W>> {
        self.matcher.final_weight(state)
    }

    fn match_type(&self) -> MatchType {
        self.matcher.match_type()
    }

    fn flags(&self) -> MatcherFlags {
        self.matcher.flags()
    }

    fn priority(&self, state: usize) -> Result<usize> {
        self.matcher.priority(state)
    }

    fn fst(&self) -> &Arc<Self::F> {
        self.matcher.fst()
    }
}

#[derive(Clone, Debug)]
struct CompactSet<K> {
    set: BTreeSet<K>,
    min_key: K,
    max_key: K,
    no_key: K,
}

impl<K: Copy + Ord + AddAssign<usize> + SubAssign<usize> + Add<usize, Output = K>> CompactSet<K> {
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

    pub fn erase(&mut self, key: K) {
        self.set.remove(&key);
        if self.set.is_empty() {
            self.min_key = self.no_key;
            self.max_key = self.no_key;
        } else if key == self.min_key {
            self.min_key += 1;
        } else if key == self.max_key {
            self.max_key -= 1;
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

    pub fn contains(&self, key: &K) -> bool {
        if self.min_key == self.no_key || *key < self.min_key || *key > self.max_key {
            // out of range
            false
        } else if self.min_key != self.no_key && self.max_key + 1 == self.min_key + self.set.len() {
            // dense range
            true
        } else {
            self.set.contains(key)
        }
    }
}
