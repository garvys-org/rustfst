use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;

use anyhow::Result;

use crate::algorithms::cache::{CacheImpl, FstImpl, StateTable};
use crate::algorithms::factor_weight::Element;
use crate::algorithms::factor_weight::{FactorIterator, FactorWeightOptions, FactorWeightType};
use crate::fst_traits::Fst;
use crate::semirings::{Semiring, WeightQuantize};
use crate::{StateId, Tr, Trs};

#[derive(Clone)]
pub struct FactorWeightImpl<W: Semiring, F: Fst<W>, B: Borrow<F>, FI: FactorIterator<W>> {
    opts: FactorWeightOptions,
    cache_impl: CacheImpl<W>,
    state_table: StateTable<Element<W>>,
    fst: B,
    unfactored: RefCell<HashMap<StateId, StateId>>,
    ghost: PhantomData<FI>,
    f: PhantomData<F>,
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
            for tr in self.fst.borrow().get_trs(old_state)?.trs() {
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
            let one = W::one();
            let weight = match elt.state {
                None => elt.weight,
                Some(s) => elt
                    .weight
                    .times(self.fst.borrow().final_weight(s)?.unwrap_or_else(|| one))
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
                    weight: W::one(),
                });
                Ok(Some(new_state))
            }
        }
    }

    fn compute_final(&mut self, state: usize) -> Result<Option<W>> {
        let zero = W::zero();
        let elt = self.state_table.find_tuple(state);
        let weight = match elt.state {
            None => elt.weight.clone(),
            Some(s) => elt
                .weight
                .times(self.fst.borrow().final_weight(s)?.unwrap_or_else(|| zero))
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
            f: PhantomData,
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
