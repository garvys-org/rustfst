use std::collections::HashMap;

use anyhow::Result;

use crate::algorithms::cache::{CacheImpl, FstImpl, StateTable};
use crate::algorithms::determinize::divisors::CommonDivisor;
use crate::algorithms::determinize::{
    DeterminizeElement, DeterminizeStateTuple, DeterminizeTr, WeightedSubset,
};
use crate::fst_traits::{Fst, MutableFst};
use crate::semirings::{DivideType, WeaklyDivisibleSemiring, WeightQuantize};
use crate::{Label, Semiring, StateId, Tr, Trs, KDELTA};
use std::collections::hash_map::Entry;
use std::marker::PhantomData;

#[derive(PartialEq, Debug)]
pub struct DeterminizeFsaImpl<'a, 'b, W: Semiring, F: Fst<W>, CD: CommonDivisor<W>> {
    fst: &'a F,
    cache_impl: CacheImpl<W>,
    state_table: StateTable<DeterminizeStateTuple<W>>,
    ghost: PhantomData<CD>,
    in_dist: Option<&'b [W]>,
    out_dist: Vec<W>,
}

impl<'a, 'b, W, F: Fst<W>, CD: CommonDivisor<W>> FstImpl for DeterminizeFsaImpl<'a, 'b, W, F, CD>
where
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
{
    type W = W;
    fn cache_impl_mut(&mut self) -> &mut CacheImpl<W> {
        &mut self.cache_impl
    }
    fn cache_impl_ref(&self) -> &CacheImpl<W> {
        &self.cache_impl
    }

    fn expand(&mut self, state: usize) -> Result<()> {
        // GetLabelMap
        let mut label_map: HashMap<Label, DeterminizeTr<W>> = HashMap::new();
        let src_tuple = self.state_table.find_tuple(state);
        for src_elt in src_tuple.subset.iter() {
            for tr in self.fst.get_trs(src_elt.state)?.trs() {
                let r = src_elt.weight.times(&tr.weight)?;

                let dest_elt = DeterminizeElement::new(tr.nextstate, r);

                // Filter Tr
                match label_map.entry(tr.ilabel) {
                    Entry::Occupied(_) => {}
                    Entry::Vacant(e) => {
                        e.insert(DeterminizeTr::from_tr(tr, 0));
                    }
                };

                label_map
                    .get_mut(&tr.ilabel)
                    .unwrap()
                    .dest_tuple
                    .subset
                    .pairs
                    .push(dest_elt);
            }
        }
        drop(src_tuple);

        for det_tr in label_map.values_mut() {
            self.norm_tr(det_tr)?;
        }

        for det_tr in label_map.values() {
            self.add_tr(state, det_tr)?;
        }

        Ok(())
    }

    fn compute_start(&mut self) -> Result<Option<usize>> {
        if let Some(start_state) = self.fst.start() {
            let elt = DeterminizeElement::new(start_state, W::one());
            let tuple = DeterminizeStateTuple {
                subset: WeightedSubset::from_vec(vec![elt]),
                filter_state: start_state,
            };
            return Ok(Some(self.find_state(&tuple)?));
        }
        Ok(None)
    }

    fn compute_final(&mut self, state: usize) -> Result<Option<W>> {
        let tuple = self.state_table.find_tuple(state);
        let mut final_weight = W::zero();
        for det_elt in tuple.subset.iter() {
            final_weight.plus_assign(
                det_elt.weight.times(
                    self.fst
                        .final_weight(det_elt.state)?
                        .unwrap_or_else(W::zero),
                )?,
            )?;
        }
        if final_weight.is_zero() {
            Ok(None)
        } else {
            Ok(Some(final_weight))
        }
    }
}

impl<'a, 'b, W, F: Fst<W>, CD: CommonDivisor<W>> DeterminizeFsaImpl<'a, 'b, W, F, CD>
where
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
{
    pub fn new(fst: &'a F, in_dist: Option<&'b [W]>) -> Result<Self> {
        if !fst.is_acceptor() {
            bail!("DeterminizeFsaImpl : expected acceptor as argument");
        }
        Ok(Self {
            fst,
            cache_impl: CacheImpl::new(),
            state_table: StateTable::new(),
            ghost: PhantomData,
            in_dist,
            out_dist: vec![],
        })
    }

    fn add_tr(&mut self, state: StateId, det_tr: &DeterminizeTr<W>) -> Result<()> {
        let tr = Tr::new(
            det_tr.label,
            det_tr.label,
            det_tr.weight.clone(),
            self.find_state(&det_tr.dest_tuple)?,
        );
        self.cache_impl.push_tr(state, tr)
    }

    fn norm_tr(&mut self, det_tr: &mut DeterminizeTr<W>) -> Result<()> {
        det_tr
            .dest_tuple
            .subset
            .pairs
            .sort_by(|a, b| a.state.cmp(&b.state));

        for dest_elt in det_tr.dest_tuple.subset.pairs.iter() {
            det_tr.weight = CD::common_divisor(&det_tr.weight, &dest_elt.weight)?;
        }

        let mut new_pairs = HashMap::new();
        for x in &mut det_tr.dest_tuple.subset.pairs {
            match new_pairs.entry(x.state) {
                Entry::Vacant(e) => {
                    e.insert(x.clone());
                }
                Entry::Occupied(mut e) => {
                    e.get_mut().weight.plus_assign(&x.weight)?;
                }
            };
        }

        det_tr.dest_tuple.subset.pairs = new_pairs.values().cloned().collect();

        for dest_elt in det_tr.dest_tuple.subset.pairs.iter_mut() {
            dest_elt.weight = dest_elt
                .weight
                .divide(&det_tr.weight, DivideType::DivideLeft)?;
            dest_elt.weight.quantize_assign(KDELTA)?;
        }

        Ok(())
    }

    fn find_state(&mut self, tuple: &DeterminizeStateTuple<W>) -> Result<StateId> {
        let s = self.state_table.find_id_from_ref(&tuple);
        if let Some(_in_dist) = self.in_dist.as_ref() {
            if self.out_dist.len() <= s {
                self.out_dist.push(self.compute_distance(&tuple.subset)?);
            }
        }
        Ok(s)
    }

    fn compute_distance(&self, subset: &WeightedSubset<W>) -> Result<W> {
        let mut outd = W::zero();
        let weight_zero = W::zero();
        for element in subset.iter() {
            let ind = if element.state < self.in_dist.as_ref().unwrap().len() {
                &self.in_dist.as_ref().unwrap()[element.state]
            } else {
                &weight_zero
            };
            outd.plus_assign(element.weight.times(ind)?)?;
        }
        Ok(outd)
    }

    pub fn compute_with_distance<F2: MutableFst<W>>(&mut self) -> Result<(F2, Vec<W>)> {
        let dfst = self.compute()?;
        let out_dist = self.out_dist.clone();
        Ok((dfst, out_dist))
    }
}
