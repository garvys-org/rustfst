use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};

use bimap::BiHashMap;

use bitflags::bitflags;

use failure::Fallible;

use crate::algorithms::weight_convert;
use crate::algorithms::weight_converters::{FromGallicConverter, ToGallicConverter};
use crate::arc::Arc;
use crate::fst_impls::VectorFst;
use crate::fst_traits::{CoreFst, ExpandedFst, Fst, MutableFst};
use crate::semirings::{DivideType, Semiring, WeaklyDivisibleSemiring};
use crate::semirings::{GallicWeight, GallicWeightMin, GallicWeightRestrict};
use crate::EPS_LABEL;
use crate::{Label, StateId};
use std::marker::PhantomData;
use std::slice::Iter as IterSlice;
use std::slice::IterMut as IterSliceMut;

bitflags! {
    struct FactorWeightOptions: u32 {
        const FACTOR_FINAL_WEIGHTS = 0b01;
        const FACTOR_ARC_WEIGHTS = 0b10;
    }
}

trait FactorIterator {
    type Weight: Semiring;

    fn new(weight: &Self::Weight) -> Self;
    fn next(&mut self) -> Option<(Self::Weight, Self::Weight)>;
}

pub struct IdentityFactor<W> {
    ghost: PhantomData<W>,
}

impl<W: Semiring> FactorIterator for IdentityFactor<W> {
    type Weight = W;

    fn new(weight: &Self::Weight) -> Self {
        Self { ghost: PhantomData }
    }

    fn next(&mut self) -> Option<(Self::Weight, Self::Weight)> {
        None
    }
}

struct CacheState<W: Semiring> {
    arcs: Vec<Arc<W>>,
    final_weight: Option<W>,
    expanded: bool
}

impl<W: Semiring> CacheState<W> {
    pub fn new() -> Self {
        Self {
            arcs: Vec::new(),
            final_weight: None,
            expanded: false,
        }
    }

    pub fn expanded(&self) -> bool {
        self.expanded
    }

    pub fn mark_expanded(&mut self) {
        self.expanded = true;
    }

    pub fn set_final_weight(&mut self, final_weight: Option<W>) {
        self.final_weight = final_weight;
    }

    pub fn final_weight(&self) -> Option<&W> {
        self.final_weight.as_ref()
    }

    pub fn push_arc(&mut self, arc: Arc<W>) {
        self.arcs.push(arc);
    }

    pub fn reserve_arcs(&mut self, n: usize) {
        self.arcs.reserve(n);
    }

    pub fn num_arcs(&self) -> usize {
        self.arcs.len()
    }

    pub fn get_arc_unchecked(&self, n: usize) -> &Arc<W> {
        unsafe { self.arcs.get_unchecked(n) }
    }

    pub fn get_arc_unchecked_mut(&mut self, n: usize) -> &mut Arc<W> {
        unsafe { self.arcs.get_unchecked_mut(n) }
    }

    pub fn iter_arcs(&self) -> IterSlice<Arc<W>> {
        self.arcs.iter()
    }

    pub fn iter_arcs_mut(&mut self) -> IterSliceMut<Arc<W>> {
        self.arcs.iter_mut()
    }
}

struct VectorCacheState<W: Semiring> {
    cache_states: Vec<CacheState<W>>,
}

impl<W: Semiring> VectorCacheState<W> {
    pub fn new() -> Self {
        Self {
            cache_states: Vec::new(),
        }
    }

    pub fn resize(&mut self, new_len: usize) {
        self.cache_states.resize_with(new_len, CacheState::new);
    }

    pub fn resize_if_necessary(&mut self, new_len: usize) {
        if self.cache_states.len() < new_len {
            self.resize(new_len)
        }
    }

    pub fn get_cache_state_unchecked(&self, state: StateId) -> &CacheState<W> {
        unsafe { self.cache_states.get_unchecked(state) }
    }

    pub fn get_cache_state_unchecked_mut(&mut self, state: StateId) -> &mut CacheState<W> {
        unsafe { self.cache_states.get_unchecked_mut(state) }
    }

    pub fn set_final_weight_unchecked(&mut self, state: StateId, final_weight: Option<W>) {
        self.get_cache_state_unchecked_mut(state).set_final_weight(final_weight);
    }

    pub fn push_arc(&mut self, state: StateId, arc: Arc<W>) {
        self.get_cache_state_unchecked_mut(state).push_arc(arc)
    }

    pub fn iter_arcs_unchecked(&self, state: StateId) -> IterSlice<Arc<W>> {
        self.get_cache_state_unchecked(state).iter_arcs()
    }

    pub fn mark_expanded_unchecked(&mut self, state: StateId) {
        self.get_cache_state_unchecked_mut(state).mark_expanded()
    }

    pub fn reserve_arcs_unchecked(&mut self, state: StateId, n: usize) {
        self.get_cache_state_unchecked_mut(state).reserve_arcs(n)
    }

    pub fn expanded(&self, state: StateId) -> bool {
        if state >= self.cache_states.len() {
            return false;
        }
        return self.get_cache_state_unchecked(state).expanded()
    }

    pub fn final_weight_unchecked(&self, state: StateId) -> Option<&W> {
        self.get_cache_state_unchecked(state).final_weight()
    }
}

struct CacheImpl<W: Semiring> {
    start_computed: bool,
    cache_start_state: Option<StateId>,
    vector_cache_states: VectorCacheState<W>,
}

impl<W: Semiring> CacheImpl<W> {
    fn new() -> Self {
        Self {
            start_computed: false,
            cache_start_state: None,
            vector_cache_states: VectorCacheState::new(),
        }
    }

    fn set_start(&mut self, start_state: Option<StateId>) {
        self.cache_start_state = start_state;
        self.start_computed = true;
        if let Some(s) = start_state {
            self.vector_cache_states.resize_if_necessary(s + 1);
        }
    }

    fn start(&self) -> Fallible<Option<StateId>> {
        if !self.start_computed {
            bail!("Can't call start() before set_start()");
        }
        Ok(self.cache_start_state)
    }

    fn set_final_weight(&mut self, state: StateId, final_weight: Option<W>) -> Fallible<()> {
        if self.vector_cache_states.expanded(state) {
            bail!("Can't modify the final weight of a fully expanded state")
        }
        self.vector_cache_states.resize_if_necessary(state + 1);
        self.vector_cache_states.set_final_weight_unchecked(state, final_weight);
        Ok(())
    }

    fn final_weight(&self, state: StateId) -> Fallible<Option<&W>> {
        if !self.vector_cache_states.expanded(state) {
            bail!("Can't call final_weight() if the state is not fully expanded")
        }
        Ok(self.vector_cache_states.final_weight_unchecked(state))
    }

    fn push_arc(&mut self, state: StateId, arc: Arc<W>) -> Fallible<()> {
        if self.vector_cache_states.expanded(state) {
            bail!("Can't add arcs to a fully expanded state")
        }
        self.vector_cache_states.resize_if_necessary(state + 1);
        self.vector_cache_states.push_arc(state, arc);
        Ok(())
    }

    fn expanded(&self, state: StateId) -> bool {
        self.vector_cache_states.expanded(state)
    }

    fn mark_expanded(&mut self, state: StateId) {
        self.vector_cache_states.resize_if_necessary(state + 1);
        self.vector_cache_states.mark_expanded_unchecked(state)
    }

    pub fn iter_arcs(&self, state: StateId) -> Fallible<IterSlice<Arc<W>>> {
        if !self.vector_cache_states.expanded(state) {
            bail!("Can't iterate arcs on a not fully expanded state")
        }
        Ok(self.vector_cache_states.iter_arcs_unchecked(state))
    }

    pub fn start_computed(&self) -> bool {
        self.start_computed
    }
}

#[derive(PartialOrd, PartialEq, Hash, Clone, Debug, Eq)]
struct Element<W: Semiring> {
    state: Option<StateId>,
    weight: W,
}

impl<W: Semiring> Element<W> {
    fn new(state: Option<StateId>, weight: W) -> Self {
        Self { state, weight }
    }
}

//use std::collections::HashMap;
struct FactorWeightImpl<'a, F: Fst> {
    opts: FactorWeightOptions,
    cache_impl: CacheImpl<F::W>,
    element_map: BiHashMap<StateId, Element<F::W>>,
    fst: &'a F,
    unfactored: HashMap<StateId, StateId>
}

impl<'a, F: Fst> FactorWeightImpl<'a, F> {
    pub fn new(fst: &'a F, opts: FactorWeightOptions) -> Fallible<Self> {
        if opts.is_empty() {
            bail!("Factoring neither arc weights nor final weights");
        }
        Ok(Self {
            opts,
            fst,
            element_map: BiHashMap::new(),
            cache_impl: CacheImpl::new(),
            unfactored: HashMap::new(),
        })
    }

    pub fn find_state(&mut self, elt: &Element<F::W>) -> StateId {
        if !(self.opts.intersects(FactorWeightOptions::FACTOR_ARC_WEIGHTS)) && elt.weight.is_one() && elt.state.is_some() {
            let old_state = elt.state.unwrap();
            if !self.unfactored.contains_key(&elt.state.unwrap()) {
                let new_state = self.element_map.len();
                self.unfactored.insert(old_state, new_state);
                self.element_map.insert(new_state, elt.clone());
            }
            return *self.unfactored.get(&old_state).unwrap();
        } else {
            if !self.element_map.contains_right(&elt) {
                let new_state = self.element_map.len();
                self.element_map.insert(new_state, elt.clone());
            }
            return *self.element_map.get_by_right(&elt).unwrap();
        }
    }

    pub fn start(&mut self) -> Option<StateId> {
        if !self.cache_impl.start_computed() {
            match self.fst.start() {
                None => self.cache_impl.set_start(None),
                Some(s) => {
                    let new_state = self.find_state(&Element{state: Some(s), weight: F::W::one()});
                    self.cache_impl.set_start(Some(new_state));
                }
            };
        }
        self.cache_impl.start().unwrap()
    }

    pub fn final_weight(&mut self, state: StateId) -> Option<&F::W> {
        if !self.cache_impl.expanded(state) {
            let elt = self.element_map.get_by_left(&state);
            unimplemented!()
        }
        self.cache_impl.final_weight(state).unwrap()
    }
}

fn factor_weight<F1, F2, FI>(fst_in: &F1) -> Fallible<F2>
where
    F1: Fst,
    F2: MutableFst<W = F1::W>,
    FI: FactorIterator<Weight = F1::W>,
{
    unimplemented!()
}
