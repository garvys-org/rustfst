use std::collections::HashMap;

use anyhow::Result;

use crate::algorithms::cache::{CacheImpl, FstImpl};
use crate::algorithms::determinize::divisors::CommonDivisor;
use crate::algorithms::determinize::{
    DeterminizeElement, DeterminizeStateTuple, DeterminizeTr, WeightedSubset,
};
use crate::algorithms::lazy_fst_revamp::{FstOp, StateTable};
use crate::fst_traits::{Fst, MutableFst};
use crate::semirings::{DivideType, WeaklyDivisibleSemiring, WeightQuantize};
use crate::{Label, Semiring, StateId, Tr, Trs, TrsVec, KDELTA};
use std::collections::hash_map::Entry;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct OutDistances<W: Semiring>(Mutex<Vec<W>>);

impl<W: Semiring> OutDistances<W> {
    fn new() -> Self {
        Self (Mutex::new(vec![]))
    }

    pub fn insert_if_needed<F: Fn() -> Result<W>>(&self, s: StateId, f: F) -> Result<()> {
        let mut v = self.0.lock().unwrap();
        if v.len() <= s {
            v.push(f()?);
        }
        Ok(())
    }
}



#[derive(Debug)]
pub struct DeterminizeFsaOp<W: Semiring, F: Fst<W>, CD: CommonDivisor<W>> {
    fst: Arc<F>,
    state_table: StateTable<DeterminizeStateTuple<W>>,
    ghost: PhantomData<CD>,
    // Distance to final NFA states.
    in_dist: Option<Arc<Vec<W>>>,
    // Distance to final DFA states.
    out_dist: OutDistances<W>,
}

impl<W, F: Fst<W>, CD: CommonDivisor<W>> FstOp<W> for DeterminizeFsaOp<W, F, CD>
where
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
{
    fn compute_start(&self) -> Result<Option<usize>> {
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

    fn compute_trs(&self, state: usize) -> Result<TrsVec<W>> {
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

        let mut trs = vec![];
        for det_tr in label_map.values() {
            trs.push(Tr::new(
                det_tr.label,
                det_tr.label,
                det_tr.weight.clone(),
                self.find_state(&det_tr.dest_tuple)?,
            ));
        }

        Ok(TrsVec(Arc::new(trs)))
    }

    fn compute_final_weight(&self, state: usize) -> Result<Option<W>> {
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

impl<W, F: Fst<W>, CD: CommonDivisor<W>> DeterminizeFsaOp<W, F, CD>
where
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
{
    pub fn new(fst: Arc<F>, in_dist: Option<Arc<Vec<W>>>) -> Result<Self> {
        if !fst.is_acceptor() {
            bail!("DeterminizeFsaImpl : expected acceptor as argument");
        }
        Ok(Self {
            fst,
            state_table: StateTable::new(),
            ghost: PhantomData,
            in_dist,
            out_dist: OutDistances::new(),
        })
    }

    fn norm_tr(&self, det_tr: &mut DeterminizeTr<W>) -> Result<()> {
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

    fn find_state(&self, tuple: &DeterminizeStateTuple<W>) -> Result<StateId> {
        let s = self.state_table.find_id_from_ref(&tuple);
        if self.in_dist.is_some() {
            self.out_dist.insert_if_needed(s, || self.compute_distance(&tuple.subset));
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

    pub fn out_dist(self) -> Vec<W> {
        self.out_dist.0.into_inner().unwrap()
    }
}
