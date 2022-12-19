use std::borrow::Borrow;
use std::marker::PhantomData;

use anyhow::Result;

use crate::algorithms::lazy::FstOp;
use crate::fst_properties::FstProperties;
use crate::fst_traits::Fst;
use crate::{Semiring, StateId, Trs, TrsVec};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

pub struct RelabelFstOp<W: Semiring, F: Fst<W>, B: Borrow<F>> {
    fst: B,
    map_ilabels: HashMap<StateId, StateId>,
    map_olabels: HashMap<StateId, StateId>,
    ghost: PhantomData<(W, F)>,
}

impl<W: Semiring, F: Fst<W>, B: Borrow<F>> RelabelFstOp<W, F, B> {
    pub fn new(
        fst: B,
        map_ilabels: HashMap<StateId, StateId>,
        map_olabels: HashMap<StateId, StateId>,
    ) -> Self {
        Self {
            fst,
            map_ilabels,
            map_olabels,
            ghost: PhantomData,
        }
    }
}

impl<W: Semiring, F: Fst<W>, B: Borrow<F>> Debug for RelabelFstOp<W, F, B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RelabelFstOp {{ fst: {:?}, map_ilabels: {:?}, map_olabels: {:?} }}",
            self.fst.borrow(),
            self.map_ilabels,
            self.map_olabels
        )
    }
}

impl<W: Semiring, F: Fst<W>, B: Borrow<F>> FstOp<W> for RelabelFstOp<W, F, B> {
    fn compute_start(&self) -> Result<Option<u32>> {
        Ok(self.fst.borrow().start())
    }

    fn compute_trs(&self, id: u32) -> Result<TrsVec<W>> {
        let trs_original = self.fst.borrow().get_trs(id)?;
        let mut trs = vec![];
        for tr in trs_original.trs() {
            let mut new_tr = tr.clone();
            if let Some(new_ilabel) = self.map_ilabels.get(&tr.ilabel) {
                new_tr.ilabel = *new_ilabel;
            }
            if let Some(new_olabel) = self.map_olabels.get(&tr.olabel) {
                new_tr.olabel = *new_olabel;
            }
            trs.push(new_tr);
        }
        Ok(TrsVec(Arc::new(trs)))
    }

    fn compute_final_weight(&self, id: u32) -> Result<Option<W>> {
        self.fst.borrow().final_weight(id)
    }

    fn properties(&self) -> FstProperties {
        unimplemented!()
    }
}
