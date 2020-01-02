use std::collections::HashMap;
use std::marker::PhantomData;

use failure::Fallible;

use bitflags::bitflags;

use crate::algorithms::cache::{CacheImpl, FstImpl, StateTable};
use crate::arc::Arc;
use crate::fst_traits::{CoreFst, ExpandedFst, Fst, MutableFst};
use crate::semirings::{Semiring, WeightQuantize};
use crate::KDELTA;
use crate::{Label, StateId};

bitflags! {
    pub struct FactorWeightType: u32 {
        const FACTOR_FINAL_WEIGHTS = 0b01;
        const FACTOR_ARC_WEIGHTS = 0b10;
    }
}

#[cfg(test)]
impl FactorWeightType {
    pub fn from_bools(factor_final_weights: bool, factor_arc_weights: bool) -> FactorWeightType {
        match (factor_final_weights, factor_arc_weights) {
            (true, true) => {
                FactorWeightType::FACTOR_FINAL_WEIGHTS | FactorWeightType::FACTOR_ARC_WEIGHTS
            }
            (true, false) => FactorWeightType::FACTOR_FINAL_WEIGHTS,
            (false, true) => FactorWeightType::FACTOR_ARC_WEIGHTS,
            (false, false) => Self::empty(),
        }
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

impl FactorWeightOptions {
    #[allow(unused)]
    pub fn new(mode: FactorWeightType) -> FactorWeightOptions {
        FactorWeightOptions {
            delta: KDELTA,
            mode,
            final_ilabel: 0,
            final_olabel: 0,
            increment_final_ilabel: false,
            increment_final_olabel: false,
        }
    }
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
    state_table: StateTable<Element<F::W>>,
    fst: &'a F,
    unfactored: HashMap<StateId, StateId>,
    ghost: PhantomData<FI>,
}

impl<'a, F: Fst, FI: FactorIterator<F::W>> FstImpl<F::W> for FactorWeightImpl<'a, F, FI>
where
    F::W: WeightQuantize + 'static,
{
    fn cache_impl(&mut self) -> &mut CacheImpl<<F as CoreFst>::W> {
        &mut self.cache_impl
    }

    fn expand(&mut self, state: usize) -> Fallible<()> {
        let elt = self.state_table.find_tuple(state).clone();
        if let Some(old_state) = elt.state {
            for arc in self.fst.arcs_iter(old_state)? {
                let weight = elt.weight.times(&arc.weight).unwrap();
                let factor_it = FI::new(weight.clone());
                if !self.factor_arc_weights() || factor_it.done() {
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
            && (elt.state.is_none() || self.fst.is_final(elt.state.unwrap())?)
        {
            let one = F::W::one();
            let weight = match elt.state {
                None => elt.weight.clone(),
                Some(s) => elt
                    .weight
                    .times(self.fst.final_weight(s)?.unwrap_or_else(|| &one))
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
        Ok(())
    }

    fn compute_start(&mut self) -> Fallible<Option<usize>> {
        match self.fst.start() {
            None => Ok(None),
            Some(s) => {
                let new_state = self.find_state(&Element {
                    state: Some(s),
                    weight: F::W::one(),
                });
                Ok(Some(new_state))
            }
        }
    }

    fn compute_final(&mut self, state: usize) -> Fallible<Option<<F as CoreFst>::W>> {
        let zero = F::W::zero();
        let elt = self.state_table.find_tuple(state);
        let weight = match elt.state {
            None => elt.weight.clone(),
            Some(s) => elt
                .weight
                .times(self.fst.final_weight(s)?.unwrap_or_else(|| &zero))
                .unwrap(),
        };
        let factor_iterator = FI::new(weight.clone());
        if !weight.is_zero() && (!self.factor_final_weights() || factor_iterator.done()) {
            Ok(Some(weight))
        } else {
            Ok(None)
        }
    }
}

impl<'a, F: Fst, FI: FactorIterator<F::W>> FactorWeightImpl<'a, F, FI>
where
    F::W: WeightQuantize + 'static,
{
    pub fn new(fst: &'a F, opts: FactorWeightOptions) -> Fallible<Self> {
        if opts.mode.is_empty() {
            bail!("Factoring neither arc weights nor final weights");
        }
        Ok(Self {
            opts,
            fst,
            state_table: StateTable::new(),
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
                // FIXME: Avoid leaking internal implementation
                let new_state = self.state_table.table.borrow().len();
                self.unfactored.insert(old_state, new_state);
                self.state_table
                    .table
                    .borrow_mut()
                    .insert(new_state, elt.clone());
            }
            self.unfactored[&old_state]
        } else {
            self.state_table.find_id(&elt)
        }
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
