use std::cell::RefCell;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::rc::Rc;

use failure::Fallible;
use std::iter::Peekable;
use std::ops::{AddAssign, SubAssign, Add};

use bitflags::bitflags;

use crate::algorithms::matchers::MatcherFlags;
use crate::algorithms::matchers::{IterItemMatcher, MatchType, Matcher};
use crate::semirings::Semiring;
use crate::{Label, EPS_LABEL, NO_LABEL};
use nom::lib::std::collections::BTreeSet;

bitflags! {
    pub struct MultiEpsMatcherFlags: u32 {
        const MULTI_EPS_LOOP =  1u32 << 0;
        const MULTI_EPS_LIST =  1u32 << 1;
    }
}

#[derive(Debug)]
struct MultiEpsMatcher<W, M> {
    matcher: Rc<RefCell<M>>,
    flags: MultiEpsMatcherFlags,
    w: PhantomData<W>,
    multi_eps_labels: CompactSet<Label>,
}

struct IteratorMultiEpsMatcher<'fst, W: Semiring + 'fst, M: Matcher<'fst, W>> {
    iter_matcher: Option<Peekable<M::Iter>>,
    // iter_labels: Option<Peekable<std::collections::hash_set::Iter<'a, Label>>>,
    matcher: Rc<RefCell<M>>,
    ghost: PhantomData<&'fst W>,
    done: bool
}

impl<'fst, W: Semiring + 'fst, M: Matcher<'fst, W>> Clone for IteratorMultiEpsMatcher<'fst, W, M> {
    fn clone(&self) -> Self {
        unimplemented!()
        // Self {
        //     iter_matcher: self.iter_matcher.clone(),
        //     // iter_labels: self.iter_labels.clone(),
        //     matcher: Rc::clone(&self.matcher),
        //     ghost: PhantomData,
        //     done: self.done
        // }
    }
}

impl<'fst, W: Semiring + 'fst, M: Matcher<'fst, W>> Iterator
    for IteratorMultiEpsMatcher<'fst, W, M>
{
    type Item = IterItemMatcher<'fst, W>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref mut matcher_iter) = &mut self.iter_matcher {
            let res = matcher_iter.next();
            let done = res.is_none();
            unimplemented!()
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

impl<'fst, W: Semiring + 'fst, M: Matcher<'fst, W>> MultiEpsMatcher<W, M> {
    pub fn new_with_opts<IM: Into<Option<M>>>(
        fst: &'fst <Self as Matcher<'fst, W>>::F,
        match_type: MatchType,
        flags: MultiEpsMatcherFlags,
        matcher: IM,
    ) -> Fallible<Self> {
        let matcher = matcher
            .into()
            .unwrap_or_else(|| M::new(fst, match_type).unwrap());
        Ok(Self {
            matcher: Rc::new(RefCell::new(matcher)),
            flags,
            w: PhantomData,
            multi_eps_labels: CompactSet::new(NO_LABEL),
        })
    }
}

impl<'fst, W: Semiring + 'fst, M: Matcher<'fst, W>> Matcher<'fst, W> for MultiEpsMatcher<W, M>{
    type F = M::F;
    type Iter = IteratorMultiEpsMatcher<'fst, W, M>;

    fn new(fst: &'fst Self::F, match_type: MatchType) -> Fallible<Self> {
        Self::new_with_opts(
            fst,
            match_type,
            MultiEpsMatcherFlags::MULTI_EPS_LOOP | MultiEpsMatcherFlags::MULTI_EPS_LIST,
            None,
        )
    }

    fn iter(&self, state: usize, label: usize) -> Fallible<Self::Iter> {
        let (iter_matcher, iter_labels) = if label == EPS_LABEL {
            (Some(self.matcher.borrow().iter(state, EPS_LABEL)?.peekable()), None)
        } else if label == NO_LABEL {
            if self.flags.contains(MultiEpsMatcherFlags::MULTI_EPS_LIST) {
                let mut iter_matcher = None;
                let mut iter_labels = self.multi_eps_labels.iter().peekable();
                while let Some(p) = iter_labels.peek() {
                    let mut it = self.matcher.borrow().iter(state, **p)?.peekable();
                    if it.peek().is_some() {
                        iter_matcher = Some(it);
                        break;
                    }
                    iter_labels.next();
                }

                if iter_labels.peek().is_some() {
                    (iter_matcher, Some(iter_labels))
                } else {
                    (Some(self.matcher.borrow().iter(state, NO_LABEL)?.peekable()), None)
                }
            } else {
                (Some(self.matcher.borrow().iter(state, NO_LABEL)?.peekable()), None)
            }
        } else if self.flags.contains(MultiEpsMatcherFlags::MULTI_EPS_LOOP) && self.multi_eps_labels.contains(&label) {
            // Empty iter
            (None, None)
        } else {
            (Some(self.matcher.borrow().iter(state, label)?.peekable()), None)
        };
        Ok(IteratorMultiEpsMatcher {
            iter_matcher,
            // iter_labels,
            matcher: Rc::clone(&self.matcher),
            ghost: PhantomData,
            done: false
        })
    }

    fn final_weight(&self, state: usize) -> Fallible<Option<&'fst W>> {
        self.matcher.borrow().final_weight(state)
    }

    fn match_type(&self) -> MatchType {
        self.matcher.borrow().match_type()
    }

    fn flags(&self) -> MatcherFlags {
        self.matcher.borrow().flags()
    }

    fn priority(&self, state: usize) -> Fallible<usize> {
        self.matcher.borrow().priority(state)
    }

    fn fst(&self) -> &'fst Self::F {
        self.matcher.borrow().fst()
    }
}

#[derive(Debug)]
struct CompactSet<K> {
    set: BTreeSet<K>,
    min_key: K,
    max_key: K,
    no_key: K,
}

impl<K: Copy + Ord + AddAssign<usize> + SubAssign<usize> + Add<usize, Output=K>> CompactSet<K> {
    pub fn new(no_key: K) -> Self {
        Self {
            set: BTreeSet::new(), min_key: no_key, max_key: no_key, no_key
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

    pub fn lower_bound(&self) -> K {
        self.min_key
    }

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