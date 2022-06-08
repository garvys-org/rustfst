use std::borrow::Borrow;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::factor_weight::{Element, FactorWeightStateTable};
use crate::algorithms::factor_weight::{FactorIterator, FactorWeightOptions, FactorWeightType};
use crate::algorithms::lazy::FstOp;
use crate::fst_properties::mutable_properties::factor_weight_properties;
use crate::fst_properties::FstProperties;
use crate::fst_traits::Fst;
use crate::semirings::{Semiring, WeightQuantize};
use crate::{StateId, Tr, Trs, TrsVec};

pub struct FactorWeightOp<W: Semiring, F: Fst<W>, B: Borrow<F>, FI: FactorIterator<W>> {
    opts: FactorWeightOptions,
    fst: B,
    fw_state_table: FactorWeightStateTable<W>,
    properties: FstProperties,
    ghost: PhantomData<FI>,
    f: PhantomData<F>,
}

impl<W: Semiring, F: Fst<W>, B: Borrow<F>, FI: FactorIterator<W>> std::fmt::Debug
    for FactorWeightOp<W, F, B, FI>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FactorWeightImpl {{ opts : {:?}, \
             fw_state_table: {:?}, fst: {:?} }}",
            self.opts,
            self.fw_state_table,
            self.fst.borrow(),
        )
    }
}

impl<W: WeightQuantize, F: Fst<W>, B: Borrow<F>, FI: FactorIterator<W>> FstOp<W>
    for FactorWeightOp<W, F, B, FI>
{
    fn compute_start(&self) -> Result<Option<StateId>> {
        match self.fst.borrow().start() {
            None => Ok(None),
            Some(s) => {
                let new_state = self.fw_state_table.find_state(&Element {
                    state: Some(s),
                    weight: W::one(),
                });
                Ok(Some(new_state))
            }
        }
    }

    fn compute_trs(&self, state: StateId) -> Result<TrsVec<W>> {
        let elt = self.fw_state_table.find_tuple(state);
        let mut trs = vec![];
        if let Some(old_state) = elt.state {
            for tr in self.fst.borrow().get_trs(old_state)?.trs() {
                let weight = elt.weight.times(&tr.weight).unwrap();
                let factor_it = FI::new(weight.clone());
                if !self.factor_tr_weights() || factor_it.done() {
                    let dest = self
                        .fw_state_table
                        .find_state(&Element::new(Some(tr.nextstate), W::one()));
                    // self.cache_impl
                    // .push_tr(state, Tr::new(tr.ilabel, tr.olabel, weight, dest))?;
                    trs.push(Tr::new(tr.ilabel, tr.olabel, weight, dest));
                } else {
                    for (p_f, p_s) in factor_it {
                        let dest = self.fw_state_table.find_state(&Element::new(
                            Some(tr.nextstate),
                            p_s.quantize(self.opts.delta)?,
                        ));
                        // self.cache_impl
                        //     .push_tr(state, Tr::new(tr.ilabel, tr.olabel, p_f, dest))?;
                        trs.push(Tr::new(tr.ilabel, tr.olabel, p_f, dest))
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
                    .times(self.fst.borrow().final_weight(s)?.unwrap_or(one))
                    .unwrap(),
            };
            let mut ilabel = self.opts.final_ilabel;
            let mut olabel = self.opts.final_olabel;
            let factor_it = FI::new(weight);
            for (p_f, p_s) in factor_it {
                let dest = self
                    .fw_state_table
                    .find_state(&Element::new(None, p_s.quantize(self.opts.delta)?));
                // self.cache_impl
                //     .push_tr(state, Tr::new(ilabel, olabel, p_f, dest))?;
                trs.push(Tr::new(ilabel, olabel, p_f, dest));
                if self.opts.increment_final_ilabel {
                    ilabel += 1;
                }
                if self.opts.increment_final_olabel {
                    olabel += 1;
                }
            }
        }
        Ok(TrsVec(Arc::new(trs)))
    }

    fn compute_final_weight(&self, state: StateId) -> Result<Option<W>> {
        let zero = W::zero();
        let elt = self.fw_state_table.find_tuple(state);
        let weight = match elt.state {
            None => elt.weight,
            Some(s) => elt
                .weight
                .times(self.fst.borrow().final_weight(s)?.unwrap_or(zero))
                .unwrap(),
        };
        let factor_iterator = FI::new(weight.clone());
        if !weight.is_zero() && (!self.factor_final_weights() || factor_iterator.done()) {
            Ok(Some(weight))
        } else {
            Ok(None)
        }
    }

    fn properties(&self) -> FstProperties {
        self.properties
    }
}

impl<W: Semiring, F: Fst<W>, B: Borrow<F>, FI: FactorIterator<W>> FactorWeightOp<W, F, B, FI>
where
    W: WeightQuantize,
{
    pub fn new(fst: B, opts: FactorWeightOptions) -> Result<Self> {
        if opts.mode.is_empty() {
            bail!("Factoring neither tr weights nor final weights");
        }
        let factor_tr_weights = opts.mode.contains(FactorWeightType::FACTOR_ARC_WEIGHTS);
        let properties = factor_weight_properties(fst.borrow().properties());
        Ok(Self {
            opts,
            fst,
            properties,
            ghost: PhantomData,
            f: PhantomData,
            fw_state_table: FactorWeightStateTable::new(factor_tr_weights),
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
}
