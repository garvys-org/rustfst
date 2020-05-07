use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

use anyhow::Result;

use bitflags::bitflags;

use crate::algorithms::cache::{CacheImpl, FstImpl, StateTable};
use crate::algorithms::lazy_fst::LazyFst;
use crate::fst_traits::{CoreFst, ExpandedFst, Fst, MutableFst};
use crate::semirings::{Semiring, WeightQuantize};
use crate::tr::Tr;
use crate::KDELTA;
use crate::{Label, StateId};

bitflags! {
    /// What kind of weight should be factored ? Tr weight ? Final weights ?
    pub struct FactorWeightType: u32 {
        /// Factor weights located on the Trs.
        const FACTOR_FINAL_WEIGHTS = 0b01;
        /// Factor weights located in the final states.
        const FACTOR_ARC_WEIGHTS = 0b10;
    }
}

#[cfg(test)]
impl FactorWeightType {
    pub fn from_bools(factor_final_weights: bool, factor_tr_weights: bool) -> FactorWeightType {
        match (factor_final_weights, factor_tr_weights) {
            (true, true) => {
                FactorWeightType::FACTOR_FINAL_WEIGHTS | FactorWeightType::FACTOR_ARC_WEIGHTS
            }
            (true, false) => FactorWeightType::FACTOR_FINAL_WEIGHTS,
            (false, true) => FactorWeightType::FACTOR_ARC_WEIGHTS,
            (false, false) => Self::empty(),
        }
    }
}

/// Configuration to control the behaviour of the `factor_weight` algorithm.
#[derive(Clone, Debug, PartialEq)]
pub struct FactorWeightOptions {
    /// Quantization delta
    pub delta: f32,
    /// Factor transition weights and/or final weights
    pub mode: FactorWeightType,
    /// Input label of transition when factoring final weights.
    pub final_ilabel: Label,
    /// Output label of transition when factoring final weights.
    pub final_olabel: Label,
    /// When factoring final w' results in > 1 trs at state, increments ilabels to make distinct ?
    pub increment_final_ilabel: bool,
    /// When factoring final w' results in > 1 trs at state, increments olabels to make distinct ?
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

/// A factor iterator takes as argument a weight w and returns a sequence of
/// pairs of weights (xi, yi) such that the sum of the products xi times yi is
/// equal to w. If w is fully factored, the iterator should return nothing.
pub trait FactorIterator<W: Semiring>:
    fmt::Debug + PartialEq + Clone + Iterator<Item = (W, W)>
{
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

#[derive(Clone)]
pub struct FactorWeightImpl<W: Semiring, F: Fst<W>, B: Borrow<F>, FI: FactorIterator<W>> {
    opts: FactorWeightOptions,
    cache_impl: CacheImpl<W>,
    state_table: StateTable<Element<W>>,
    fst: B,
    unfactored: RefCell<HashMap<StateId, StateId>>,
    ghost: PhantomData<FI>,
}

impl<W: Semiring, F: Fst<W>, B: Borrow<F>, FI: FactorIterator<W>> std::fmt::Debug
    for FactorWeightImpl<W, F, B, FI>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FactorWeightImpl {{ opts : {:?}, cache_impl: {:?}, \
             state_table: {:?}, fst: {:?}, unfactored : {:?} }}",
            self.opts,
            self.cache_impl,
            self.state_table,
            self.fst.borrow(),
            self.unfactored.borrow()
        )
    }
}

impl<W: Semiring, F: Fst<W> + PartialEq, B: Borrow<F>, FI: FactorIterator<W>> PartialEq
    for FactorWeightImpl<W, F, B, FI>
{
    fn eq(&self, other: &Self) -> bool {
        self.opts.eq(&other.opts)
            && self.cache_impl.eq(&other.cache_impl)
            && self.state_table.eq(&other.state_table)
            && self.fst.borrow().eq(&other.fst.borrow())
            && self.unfactored.borrow().eq(&other.unfactored.borrow())
    }
}

impl<W, F: Fst<W>, B: Borrow<F>, FI: FactorIterator<W>> FstImpl for FactorWeightImpl<W, F, B, FI>
where
    W: WeightQuantize,
{
    type W = W;
    fn cache_impl_mut(&mut self) -> &mut CacheImpl<W> {
        &mut self.cache_impl
    }
    fn cache_impl_ref(&self) -> &CacheImpl<W> {
        &self.cache_impl
    }

    fn expand(&mut self, state: usize) -> Result<()> {
        let elt = self.state_table.find_tuple(state).clone();
        if let Some(old_state) = elt.state {
            for tr in self.fst.borrow().tr_iter(old_state)? {
                let weight = elt.weight.times(&tr.weight).unwrap();
                let factor_it = FI::new(weight.clone());
                if !self.factor_tr_weights() || factor_it.done() {
                    let dest = self.find_state(&Element::new(Some(tr.nextstate), W::one()));
                    self.cache_impl
                        .push_tr(state, Tr::new(tr.ilabel, tr.olabel, weight, dest))?;
                } else {
                    for (p_f, p_s) in factor_it {
                        let dest = self.find_state(&Element::new(
                            Some(tr.nextstate),
                            p_s.quantize(self.opts.delta)?,
                        ));
                        self.cache_impl
                            .push_tr(state, Tr::new(tr.ilabel, tr.olabel, p_f, dest))?;
                    }
                }
            }
        }
        if self.factor_final_weights()
            && (elt.state.is_none() || self.fst.borrow().is_final(elt.state.unwrap())?)
        {
            let one = F::W::one();
            let weight = match elt.state {
                None => elt.weight,
                Some(s) => elt
                    .weight
                    .times(self.fst.borrow().final_weight(s)?.unwrap_or_else(|| &one))
                    .unwrap(),
            };
            let mut ilabel = self.opts.final_ilabel;
            let mut olabel = self.opts.final_olabel;
            let factor_it = FI::new(weight);
            for (p_f, p_s) in factor_it {
                let dest = self.find_state(&Element::new(None, p_s.quantize(self.opts.delta)?));
                self.cache_impl
                    .push_tr(state, Tr::new(ilabel, olabel, p_f, dest))?;
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

    fn compute_start(&mut self) -> Result<Option<usize>> {
        match self.fst.borrow().start() {
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

    fn compute_final(&mut self, state: usize) -> Result<Option<W>> {
        let zero = F::W::zero();
        let elt = self.state_table.find_tuple(state);
        let weight = match elt.state {
            None => elt.weight.clone(),
            Some(s) => elt
                .weight
                .times(self.fst.borrow().final_weight(s)?.unwrap_or_else(|| &zero))
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

impl<W: Semiring, F: Fst<W>, B: Borrow<F>, FI: FactorIterator<W>> FactorWeightImpl<W, F, B, FI>
where
    W: WeightQuantize,
{
    pub fn new(fst: B, opts: FactorWeightOptions) -> Result<Self> {
        if opts.mode.is_empty() {
            bail!("Factoring neither tr weights nor final weights");
        }
        Ok(Self {
            opts,
            fst,
            state_table: StateTable::new(),
            cache_impl: CacheImpl::new(),
            unfactored: RefCell::new(HashMap::new()),
            ghost: PhantomData,
        })
    }

    pub fn factor_tr_weights(&self) -> bool {
        self.opts
            .mode
            .intersects(FactorWeightType::FACTOR_ARC_WEIGHTS)
    }

    pub fn factor_final_weights(&self) -> bool {
        self.opts
            .mode
            .intersects(FactorWeightType::FACTOR_FINAL_WEIGHTS)
    }

    fn find_state(&self, elt: &Element<W>) -> StateId {
        if !self.factor_tr_weights() && elt.weight.is_one() && elt.state.is_some() {
            let old_state = elt.state.unwrap();
            if !self.unfactored.borrow().contains_key(&elt.state.unwrap()) {
                // FIXME: Avoid leaking internal implementation
                let new_state = self.state_table.table.borrow().len();
                self.unfactored.borrow_mut().insert(old_state, new_state);
                self.state_table
                    .table
                    .borrow_mut()
                    .insert(new_state, elt.clone());
            }
            self.unfactored.borrow()[&old_state]
        } else {
            self.state_table.find_id_from_ref(&elt)
        }
    }
}

/// The result of weight factoring is a transducer equivalent to the
/// input whose path weights have been factored according to the FactorIterator.
/// States and transitions will be added as necessary. The algorithm is a
/// generalization to arbitrary weights of the second step of the input
/// epsilon-normalization algorithm.
pub fn factor_weight<W, F1, B, F2, FI>(fst_in: B, opts: FactorWeightOptions) -> Result<F2>
where
    F1: Fst<W>,
    B: Borrow<F1>,
    F2: MutableFst<W>,
    FI: FactorIterator<W>,
    W: WeightQuantize,
{
    let mut factor_weight_impl: FactorWeightImpl<W, F1, B, FI> = FactorWeightImpl::new(fst_in, opts)?;
    factor_weight_impl.compute()
}

/// The result of weight factoring is a transducer equivalent to the
/// input whose path weights have been factored according to the FactorIterator.
/// States and transitions will be added as necessary. The algorithm is a
/// generalization to arbitrary weights of the second step of the input
/// epsilon-normalization algorithm. This version is a Delayed FST.
pub type FactorWeightFst<W, F, B, FI> = LazyFst<FactorWeightImpl<W, F, B, FI>>;

impl<'a, W, F: Fst<W>, B: Borrow<F>, FI: FactorIterator<W>> FactorWeightFst<W, F, B, FI>
where
    W: WeightQuantize,
{
    pub fn new(fst: B, opts: FactorWeightOptions) -> Result<Self> {
        let isymt = fst.borrow().input_symbols().cloned();
        let osymt = fst.borrow().output_symbols().cloned();
        Ok(Self::from_impl(
            FactorWeightImpl::new(fst, opts)?,
            isymt,
            osymt,
        ))
    }
}
