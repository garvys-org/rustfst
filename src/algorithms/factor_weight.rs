use std::collections::{HashMap, VecDeque, HashSet};

use bimap::BiHashMap;

use bitflags::bitflags;

use failure::Fallible;

use crate::arc::Arc;
use crate::fst_traits::{ExpandedFst, Fst, MutableFst};
use crate::semirings::{Semiring, WeightQuantize};
use crate::{Label, StateId};
use std::marker::PhantomData;
use std::slice::Iter as IterSlice;

use crate::algorithms::cache::CacheImpl;

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
    fn new(weight: W) -> Self;
    fn done(&self) -> bool;
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

    pub fn final_weight(&mut self, state: StateId) -> Fallible<Option<&F::W>> {
        if !self.cache_impl.has_final(state) {
            let elt = self.element_map.get_by_left(&state).unwrap();
            let weight = match elt.state {
                None => elt.weight.clone(),
                Some(s) => elt
                    .weight
                    .times(self.fst.final_weight(s).unwrap_or_else(F::W::zero))
                    .unwrap(),
            };
            let factor_iterator = FI::new(weight.clone());
            if !weight.is_zero() && (!self.factor_final_weights() || factor_iterator.done()) {
                self.cache_impl.set_final_weight(state, Some(weight))?;
            } else {
                self.cache_impl.set_final_weight(state, None)?;
            }
        }
        self.cache_impl.final_weight(state)
    }

    #[allow(unused)]
    pub fn num_arcs(&mut self, state: StateId) -> Fallible<usize> {
        if !self.cache_impl.expanded(state) {
            self.expand(state)?;
        }
        self.cache_impl.num_arcs(state)
    }

    pub fn arcs_iter(&mut self, state: StateId) -> Fallible<IterSlice<Arc<F::W>>> {
        if !self.cache_impl.expanded(state) {
            self.expand(state)?;
        }
        self.cache_impl.arcs_iter(state)
    }

    pub fn expand(&mut self, state: StateId) -> Fallible<()> {
        let elt = self.element_map.get_by_left(&state).unwrap().clone();
        if let Some(old_state) = elt.state {
            for arc in self.fst.arcs_iter(old_state)? {
                let weight = elt.weight.times(&arc.weight).unwrap();
                let factor_it = FI::new(weight.clone());
                if !self.factor_arc_weights() && factor_it.done() {
                    let dest = self.find_state(&Element::new(Some(arc.nextstate), F::W::one()));
                    self.cache_impl
                        .push_arc(state, Arc::new(arc.ilabel, arc.olabel, weight, dest))?;
                } else {
                    for (p_f, p_s) in factor_it {
                        let dest = self.find_state(&Element::new(
                            Some(arc.nextstate),
                            p_s.quantize(self.opts.delta)?,
                        ));
                        self.cache_impl
                            .push_arc(state, Arc::new(arc.ilabel, arc.olabel, p_f, dest))?;
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
            let factor_it = FI::new(weight);
            for (p_f, p_s) in factor_it {
                let dest = self.find_state(&Element::new(None, p_s.quantize(self.opts.delta)?));
                self.cache_impl
                    .push_arc(state, Arc::new(ilabel, olabel, p_f, dest))?;
                if self.opts.increment_final_ilabel {
                    ilabel += 1;
                }
                if self.opts.increment_final_olabel {
                    olabel += 1;
                }
            }
        }
        self.cache_impl.mark_expanded(state);
        Ok(())
    }
}

impl<'a, F: Fst, FI: FactorIterator<F::W>> FactorWeightImpl<'a, F, FI>
where
    F::W: WeightQuantize,
{
    pub fn compute<F2: MutableFst<W = F::W> + ExpandedFst<W = F::W>>(&mut self) -> Fallible<F2>
    where
        F::W: 'static,
    {
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
        let mut visited_states = HashSet::new();
        visited_states.insert(start_state);
        queue.push_back(start_state);
        while !queue.is_empty() {
            let s = queue.pop_front().unwrap();
            for arc in self.arcs_iter(s)? {
                if !visited_states.contains(&arc.nextstate) {
                    queue.push_back(arc.nextstate);
                    visited_states.insert(arc.nextstate);
                }
                let n = fst_out.num_states();
                for _ in n..=arc.nextstate {
                    fst_out.add_state();
                }
                fst_out.add_arc(s, arc.clone())?;
            }
            if let Some(f_w) = self.final_weight(s)? {
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
    let mut factor_weight_impl: FactorWeightImpl<F1, FI> = FactorWeightImpl::new(fst_in, opts)?;
    factor_weight_impl.compute()
}
