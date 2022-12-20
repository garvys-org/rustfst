use std::borrow::Borrow;
use std::marker::PhantomData;

use anyhow::Result;

use crate::algorithms::lazy::FstOp;
use crate::algorithms::tr_compares::TrCompare;
use crate::fst_properties::FstProperties;
use crate::fst_traits::Fst;
use crate::prelude::compose::LabelReachableData;
use crate::prelude::{ILabelCompare, OLabelCompare};
use crate::{Semiring, Trs, TrsVec};
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

pub struct LookaheadRelabelFstOp<W: Semiring, F: Fst<W>, B: Borrow<F>> {
    pub(crate) fst: B,
    label_reachable_data: LabelReachableData,
    relabel_input: bool,
    ghost: PhantomData<(W, F)>,
}

impl<W: Semiring, F: Fst<W>, B: Borrow<F>> LookaheadRelabelFstOp<W, F, B> {
    pub fn new(fst: B, label_reachable_data: LabelReachableData, relabel_input: bool) -> Self {
        Self {
            fst,
            label_reachable_data,
            relabel_input,
            ghost: PhantomData,
        }
    }
}

impl<W: Semiring, F: Fst<W>, B: Borrow<F>> Debug for LookaheadRelabelFstOp<W, F, B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "LookaheadRelabelFstOp {{ fst: {:?}, label_reachable_data: {:?}, relabel_input: {:?} }}", self.fst.borrow(), self.label_reachable_data, self.relabel_input)
    }
}

impl<W: Semiring, F: Fst<W>, B: Borrow<F>> FstOp<W> for LookaheadRelabelFstOp<W, F, B> {
    fn compute_start(&self) -> Result<Option<u32>> {
        Ok(self.fst.borrow().start())
    }

    fn compute_trs(&self, id: u32) -> Result<TrsVec<W>> {
        let trs = self.fst.borrow().get_trs(id)?;
        let mut new_trs = Vec::with_capacity(trs.trs().len());
        for tr in trs.trs() {
            let mut new_tr = tr.clone();
            if self.relabel_input {
                new_tr.ilabel = self.label_reachable_data.relabel_unmut(tr.ilabel)?;
            } else {
                new_tr.olabel = self.label_reachable_data.relabel_unmut(tr.olabel)?;
            }
            new_trs.push(new_tr);
        }

        if self.relabel_input {
            new_trs.sort_by(ILabelCompare::compare)
        } else {
            new_trs.sort_by(OLabelCompare::compare)
        }
        Ok(TrsVec(Arc::new(new_trs)))
    }

    fn compute_final_weight(&self, id: u32) -> Result<Option<W>> {
        self.fst.borrow().final_weight(id)
    }

    fn properties(&self) -> FstProperties {
        if self.relabel_input {
            FstProperties::I_LABEL_SORTED
        } else {
            FstProperties::O_LABEL_SORTED
        }
    }
}
