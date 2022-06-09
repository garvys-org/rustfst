use std::borrow::Borrow;
use std::collections::btree_map::Entry as EntryBTreeMap;
use std::collections::hash_map::Entry as EntryHashMap;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::determinize::divisors::CommonDivisor;
use crate::algorithms::determinize::{
    DeterminizeElement, DeterminizeStateTable, DeterminizeStateTuple, DeterminizeTr, WeightedSubset,
};
use crate::algorithms::lazy::FstOp;
use crate::fst_properties::FstProperties;
use crate::fst_traits::Fst;
use crate::semirings::{DivideType, WeaklyDivisibleSemiring, WeightQuantize};
use crate::{Label, Semiring, StateId, Tr, Trs, TrsVec};

#[derive(Debug)]
pub struct DeterminizeFsaOp<W, F, CD, B, BT>
where
    W: Semiring,
    F: Fst<W>,
    CD: CommonDivisor<W>,
    B: Borrow<F> + Debug,
    BT: Borrow<[W]> + Debug,
{
    fst: B,
    state_table: DeterminizeStateTable<W, BT>,
    delta: f32,
    ghost: PhantomData<(CD, F)>,
}

impl<W, F, CD, B, BT> FstOp<W> for DeterminizeFsaOp<W, F, CD, B, BT>
where
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
    F: Fst<W>,
    CD: CommonDivisor<W>,
    B: Borrow<F> + Debug,
    BT: Borrow<[W]> + Debug + PartialEq,
{
    fn compute_start(&self) -> Result<Option<StateId>> {
        if let Some(start_state) = self.fst.borrow().start() {
            let elt = DeterminizeElement::new(start_state, W::one());
            let tuple = DeterminizeStateTuple {
                subset: WeightedSubset::from_vec(vec![elt]),
                filter_state: start_state,
            };
            return Ok(Some(self.find_state(&tuple)?));
        }
        Ok(None)
    }

    fn compute_trs(&self, state: StateId) -> Result<TrsVec<W>> {
        // GetLabelMap
        let mut label_map: BTreeMap<Label, DeterminizeTr<W>> = BTreeMap::new();
        let src_tuple = self.state_table.find_tuple(state);
        for src_elt in src_tuple.subset.iter() {
            for tr in self.fst.borrow().get_trs(src_elt.state)?.trs() {
                let r = src_elt.weight.times(&tr.weight)?;

                let dest_elt = DeterminizeElement::new(tr.nextstate, r);

                // Filter Tr
                match label_map.entry(tr.ilabel) {
                    EntryBTreeMap::Occupied(_) => {}
                    EntryBTreeMap::Vacant(e) => {
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

    fn compute_final_weight(&self, state: StateId) -> Result<Option<W>> {
        let tuple = self.state_table.find_tuple(state);
        let mut final_weight = W::zero();
        for det_elt in tuple.subset.iter() {
            final_weight.plus_assign(
                det_elt.weight.times(
                    self.fst
                        .borrow()
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

    fn properties(&self) -> FstProperties {
        // Properties are set for the DeterminizeFst object. DeterminizeFsa shouldn't be used directly
        FstProperties::empty()
    }
}

impl<W, F, CD, B, BT> DeterminizeFsaOp<W, F, CD, B, BT>
where
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
    F: Fst<W>,
    CD: CommonDivisor<W>,
    B: Borrow<F> + Debug,
    BT: Borrow<[W]> + Debug + PartialEq,
{
    pub fn new(fst: B, in_dist: Option<BT>, delta: f32) -> Result<Self> {
        if !fst.borrow().properties().contains(FstProperties::ACCEPTOR) {
            bail!("DeterminizeFsaImpl : expected acceptor as argument");
        }
        Ok(Self {
            fst,
            state_table: DeterminizeStateTable::new(in_dist),
            delta,
            ghost: PhantomData,
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
                EntryHashMap::Vacant(e) => {
                    e.insert(x.clone());
                }
                EntryHashMap::Occupied(mut e) => {
                    e.get_mut().weight.plus_assign(&x.weight)?;
                }
            };
        }

        det_tr.dest_tuple.subset.pairs = new_pairs.values().cloned().collect();

        for dest_elt in det_tr.dest_tuple.subset.pairs.iter_mut() {
            dest_elt.weight = dest_elt
                .weight
                .divide(&det_tr.weight, DivideType::DivideLeft)?;
            dest_elt.weight.quantize_assign(self.delta)?;
        }

        Ok(())
    }

    fn find_state(&self, tuple: &DeterminizeStateTuple<W>) -> Result<StateId> {
        self.state_table.find_id_from_ref(tuple)
    }

    pub fn out_dist(self) -> Result<Vec<W>> {
        let out_dist = self.state_table.out_dist();
        out_dist
            .into_iter()
            .enumerate()
            .map(|(s, e)| {
                e.ok_or_else(|| format_err!("Outdist for state {} has not been computed", s))
            })
            .collect::<Result<Vec<_>>>()
    }
}
