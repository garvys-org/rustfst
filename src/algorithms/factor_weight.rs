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
use crate::semirings::{Semiring, WeightQuantize};
use crate::semirings::{GallicWeight, GallicWeightMin, GallicWeightRestrict};
use crate::{Label, StateId};
use crate::{EPS_LABEL, KDELTA};
use std::marker::PhantomData;
use std::slice::Iter as IterSlice;
use std::slice::IterMut as IterSliceMut;

bitflags! {
    pub struct FactorWeightType: u32 {
        const FACTOR_FINAL_WEIGHTS = 0b01;
        const FACTOR_ARC_WEIGHTS = 0b10;
    }
}

pub struct FactorWeightOptions {
    /// Quantization delta
    pub delta: f32,
    /// Factor arc weights and/or final weights
    pub mode: FactorWeightType,
    /// Input label of arc when factoring final weights.
    pub final_ilabel: Label,
    /// Output label of arc when factoring final weights.
    pub final_olabel: Label,
    /// When factoring final w' results in > 1 arcs at state, increments ilabels to make distinct ?
    pub increment_final_ilabel: bool,
    /// When factoring final w' results in > 1 arcs at state, increments olabels to make distinct ?
    pub increment_final_olabel: bool,
}

pub trait FactorIterator<W: Semiring>: Iterator<Item = (W, W)> {
    fn new(weight: &W) -> Self;
    fn done(&self) -> bool;
}

pub struct IdentityFactor<W> {
    ghost: PhantomData<W>,
}

impl<W: Semiring> FactorIterator<W> for IdentityFactor<W> {
    fn new(weight: &W) -> Self {
        Self { ghost: PhantomData }
    }

    fn done(&self) -> bool {
        true
    }
}

impl<W: Semiring> Iterator for IdentityFactor<W> {
    type Item = (W, W);

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

#[derive(Clone)]
struct CacheImplArcIterator<'a, W: Semiring> {
    arcs: &'a Vec<Arc<W>>
}

impl<'a, W: Semiring> Iterator for CacheImplArcIterator<'a, W> {
    type Item = &'a Arc<W>;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

struct CacheState<W: Semiring> {
    arcs: Vec<Arc<W>>,
    final_weight: Option<W>,
    expanded: bool,
    has_final: bool,
}

impl<W: Semiring> CacheState<W> {
    pub fn new() -> Self {
        Self {
            arcs: Vec::new(),
            final_weight: None,
            expanded: false,
            has_final: false,
        }
    }

    pub fn has_final(&self) -> bool {
        self.has_final
    }

    pub fn expanded(&self) -> bool {
        self.expanded
    }

    pub fn mark_expanded(&mut self) {
        self.expanded = true;
    }

    pub fn set_final_weight(&mut self, final_weight: Option<W>) {
        self.final_weight = final_weight;
        self.has_final = true;
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

    pub fn arcs_iter(&self) -> IterSlice<Arc<W>> {
        self.arcs.iter()
    }

    pub fn arcs_iter_mut(&mut self) -> IterSliceMut<Arc<W>> {
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
        self.get_cache_state_unchecked_mut(state)
            .set_final_weight(final_weight);
    }

    pub fn push_arc(&mut self, state: StateId, arc: Arc<W>) {
        self.get_cache_state_unchecked_mut(state).push_arc(arc)
    }

    pub fn arcs_iter_unchecked(&self, state: StateId) -> IterSlice<Arc<W>> {
        self.get_cache_state_unchecked(state).arcs_iter()
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
        return self.get_cache_state_unchecked(state).expanded();
    }

    pub fn has_final(&self, state: StateId) -> bool {
        if state >= self.cache_states.len() {
            return false;
        }
        return self.get_cache_state_unchecked(state).has_final()
    }

    pub fn final_weight_unchecked(&self, state: StateId) -> Option<&W> {
        self.get_cache_state_unchecked(state).final_weight()
    }

    pub fn num_arcs(&self, state: StateId) -> usize {
        self.get_cache_state_unchecked(state).num_arcs()
    }
}

struct CacheImpl<W: Semiring> {
    has_start: bool,
    cache_start_state: Option<StateId>,
    vector_cache_states: VectorCacheState<W>,
}

impl<W: Semiring> CacheImpl<W> {
    fn new() -> Self {
        Self {
            has_start: false,
            cache_start_state: None,
            vector_cache_states: VectorCacheState::new(),
        }
    }

    fn set_start(&mut self, start_state: Option<StateId>) {
        self.cache_start_state = start_state;
        self.has_start = true;
        if let Some(s) = start_state {
            self.vector_cache_states.resize_if_necessary(s + 1);
        }
    }

    fn start(&self) -> Fallible<Option<StateId>> {
        if !self.has_start {
            bail!("Can't call start() before set_start()");
        }
        Ok(self.cache_start_state)
    }

    fn set_final_weight(&mut self, state: StateId, final_weight: Option<W>) -> Fallible<()> {
        self.vector_cache_states.resize_if_necessary(state + 1);
        self.vector_cache_states
            .set_final_weight_unchecked(state, final_weight);
        Ok(())
    }

    fn final_weight(&self, state: StateId) -> Fallible<Option<&W>> {
        if !self.vector_cache_states.has_final(state) {
            bail!("Can't call final_weight() before set_final_weight()")
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

    fn num_arcs(&self, state: StateId) -> Fallible<usize> {
        if !self.vector_cache_states.expanded(state) {
            bail!("Can't call num_arcs on a state that is not fully expanded");
        }
        Ok(self.vector_cache_states.num_arcs(state))
    }

    fn expanded(&self, state: StateId) -> bool {
        self.vector_cache_states.expanded(state)
    }

    fn has_final(&self, state: StateId) -> bool {
        self.vector_cache_states.has_final(state)
    }

    fn mark_expanded(&mut self, state: StateId) {
        self.vector_cache_states.resize_if_necessary(state + 1);
        self.vector_cache_states.mark_expanded_unchecked(state)
    }

    pub fn arcs_iter(&self, state: StateId) -> Fallible<IterSlice<Arc<W>>> {
        if !self.vector_cache_states.expanded(state) {
            bail!("Can't iterate arcs on a not fully expanded state")
        }
        Ok(self.vector_cache_states.arcs_iter_unchecked(state))
    }

    pub fn has_start(&self) -> bool {
        self.has_start
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

struct FactorWeightImpl<'a, F: Fst, FI: FactorIterator<F::W>> {
    opts: FactorWeightOptions,
    cache_impl: CacheImpl<F::W>,
    element_map: BiHashMap<StateId, Element<F::W>>,
    fst: &'a F,
    unfactored: HashMap<StateId, StateId>,
    ghost: PhantomData<FI>,
}

impl<'a, F: Fst, FI: FactorIterator<F::W>> FactorWeightImpl<'a, F, FI>
where
    F::W: WeightQuantize,
{
    pub fn new(fst: &'a F, opts: FactorWeightOptions) -> Fallible<Self> {
        if opts.mode.is_empty() {
            bail!("Factoring neither arc weights nor final weights");
        }
        Ok(Self {
            opts,
            fst,
            element_map: BiHashMap::new(),
            cache_impl: CacheImpl::new(),
            unfactored: HashMap::new(),
            ghost: PhantomData,
        })
    }

    pub fn factor_arc_weights(&self) -> bool {
        self.opts
            .mode
            .intersects(FactorWeightType::FACTOR_ARC_WEIGHTS)
    }

    pub fn factor_final_weights(&self) -> bool {
        self.opts
            .mode
            .intersects(FactorWeightType::FACTOR_FINAL_WEIGHTS)
    }

    pub fn find_state(&mut self, elt: &Element<F::W>) -> StateId {
        if !self.factor_arc_weights() && elt.weight.is_one() && elt.state.is_some() {
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
        if !self.cache_impl.has_start() {
            match self.fst.start() {
                None => self.cache_impl.set_start(None),
                Some(s) => {
                    let new_state = self.find_state(&Element {
                        state: Some(s),
                        weight: F::W::one(),
                    });
                    self.cache_impl.set_start(Some(new_state));
                }
            };
        }
        self.cache_impl.start().unwrap()
    }

    pub fn final_weight(&mut self, state: StateId) -> Option<&F::W> {
        if !self.cache_impl.has_final(state) {
            let elt = self.element_map.get_by_left(&state).unwrap();
            let weight = match elt.state {
                None => elt.weight.clone(),
                Some(s) => elt
                    .weight
                    .times(self.fst.final_weight(s).unwrap_or_else(F::W::one))
                    .unwrap(),
            };
            let mut factor_iterator = FI::new(&weight);
            if !(self
                .opts
                .mode
                .intersects(FactorWeightType::FACTOR_FINAL_WEIGHTS))
                || factor_iterator.done()
            {
                self.cache_impl.set_final_weight(state, Some(weight));
            } else {
                self.cache_impl.set_final_weight(state, None);
            }
        }
        self.cache_impl.final_weight(state).unwrap()
    }

    pub fn num_arcs(&mut self, state: StateId) -> Fallible<usize> {
        if !self.cache_impl.expanded(state) {
            self.expand(state);
        }
        self.cache_impl.num_arcs(state)
    }

    pub fn arcs_iter(&mut self, state: StateId) -> Fallible<IterSlice<Arc<F::W>>> {
        if !self.cache_impl.expanded(state) {
            self.expand(state);
        }
        self.cache_impl.arcs_iter(state)
    }

    pub fn expand(&mut self, state: StateId) {
        let elt = self.element_map.get_by_left(&state).unwrap().clone();
        if let Some(old_state) = elt.state {
            for arc in self.fst.arcs_iter(old_state).unwrap() {
                let weight = elt.weight.times(&arc.weight).unwrap();
                let mut factor_it = FI::new(&weight);
                if !self.factor_arc_weights() && factor_it.done() {
                    let dest = self.find_state(&Element::new(Some(arc.nextstate), F::W::one()));
                    self.cache_impl
                        .push_arc(state, Arc::new(arc.ilabel, arc.olabel, weight, dest));
                } else {
                    for (p_f, p_s) in factor_it {
                        let dest = self.find_state(&Element::new(
                            Some(arc.nextstate),
                            p_s.quantize(self.opts.delta).unwrap(),
                        ));
                        self.cache_impl
                            .push_arc(state, Arc::new(arc.ilabel, arc.olabel, p_f, dest));
                    }
                }
            }
        }
        if self.factor_final_weights()
            && (elt.state.is_none() || self.fst.is_final(elt.state.unwrap()))
        {
            let weight = match elt.state {
                None => elt.weight.clone(),
                Some(s) => elt
                    .weight
                    .times(self.fst.final_weight(s).unwrap_or_else(F::W::one))
                    .unwrap(),
            };
            let mut ilabel = self.opts.final_ilabel;
            let mut olabel = self.opts.final_olabel;
            let mut factor_it = FI::new(&weight);
            for (p_f, p_s) in factor_it {
                let dest =
                    self.find_state(&Element::new(None, p_s.quantize(self.opts.delta).unwrap()));
                self.cache_impl
                    .push_arc(state, Arc::new(ilabel, olabel, p_f, dest));
                if self.opts.increment_final_ilabel {
                    ilabel += 1;
                }
                if self.opts.increment_final_olabel {
                    olabel += 1;
                }
            }
        }
        self.cache_impl.mark_expanded(state);
    }
}

impl<'a, F: Fst, FI: FactorIterator<F::W>> FactorWeightImpl<'a, F, FI>
    where
        F::W: WeightQuantize
{
    pub fn compute<F2: MutableFst<W=F::W> + ExpandedFst<W=F::W>>(&mut self) -> Fallible<F2> where F::W: 'static{
        let start_state = self.start();
        let mut fst_out = F2::new();
        if start_state.is_none() {
            return Ok(fst_out);
        }
        let start_state = start_state.unwrap();
        for _ in 0..=start_state {
            fst_out.add_state();
        }
        fst_out.set_start(start_state)?;
        let mut queue = VecDeque::new();
        while !queue.is_empty() {
            let s = queue.pop_front().unwrap();
            for arc in self.arcs_iter(s)? {
                queue.push_back(arc.nextstate);
                let n = fst_out.num_states();
                for _ in n..=arc.nextstate{
                    fst_out.add_state();
                }
                fst_out.add_arc(s, arc.clone())?;
            }
            if let Some(f_w) = self.final_weight(s) {
                fst_out.set_final(s, f_w.clone())?;
            }
        }
        Ok(fst_out)
    }
}


pub fn factor_weight<F1, F2, FI>(fst_in: &F1, opts: FactorWeightOptions) -> Fallible<F2>
where
    F1: Fst,
    F2: MutableFst<W = F1::W> + ExpandedFst<W = F1::W>,
    FI: FactorIterator<F1::W>,
    F1::W: WeightQuantize + 'static,
{
    let mut factor_weight_impl : FactorWeightImpl<F1, FI> = FactorWeightImpl::new(fst_in, opts)?;
    factor_weight_impl.compute()
}
