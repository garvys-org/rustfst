use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::lazy::FstOp2;
use crate::algorithms::randgen::rand_state::RandState;
use crate::algorithms::randgen::tr_sampler::TrSampler;
use crate::algorithms::randgen::TrSelector;
use crate::fst_properties::mutable_properties::rand_gen_properties;
use crate::fst_properties::FstProperties;
use crate::prelude::Fst;
use crate::{Semiring, StateId, Tr, Trs, TrsVec, NO_STATE_ID};

pub struct RandGenFstOp<W, F, B, S>
where
    W: Semiring<Type = f32>,
    F: Fst<W>,
    B: Borrow<F>,
    S: TrSelector,
{
    fst: B,
    sampler: RefCell<TrSampler<W, F, B, S>>,
    npath: usize,
    state_table: RefCell<Vec<Rc<RandState>>>,
    weighted: bool,
    remove_total_weight: bool,
    superfinal: RefCell<StateId>,
}

impl<W, F, B, S> RandGenFstOp<W, F, B, S>
where
    W: Semiring<Type = f32>,
    F: Fst<W>,
    B: Borrow<F>,
    S: TrSelector,
{
    pub fn new(
        fst: B,
        sampler: TrSampler<W, F, B, S>,
        npath: usize,
        weighted: bool,
        remove_total_weight: bool,
    ) -> Self {
        Self {
            fst,
            sampler: RefCell::new(sampler),
            npath,
            state_table: RefCell::new(vec![]),
            weighted,
            remove_total_weight,
            superfinal: RefCell::new(NO_STATE_ID),
        }
    }
}

impl<W, F, B, S> Debug for RandGenFstOp<W, F, B, S>
where
    W: Semiring<Type = f32>,
    F: Fst<W>,
    B: Borrow<F>,
    S: TrSelector,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RandGenFstOp {{ fst : {:?}, sampler : {:?}, npath : {:?}, state_table : {:?}, weighted : {:?}, remove_total_weight : {:?}, superfinal : {:?} }}",
            self.fst.borrow(),
            self.sampler.borrow(),
            self.npath,
            self.state_table.borrow(),
            self.weighted,
            self.remove_total_weight,
            self.superfinal
        )
    }
}

impl<W, F, B, S> FstOp2<W> for RandGenFstOp<W, F, B, S>
where
    W: Semiring<Type = f32>,
    F: Fst<W>,
    B: Borrow<F>,
    S: TrSelector,
{
    fn compute_start(&self) -> Result<Option<StateId>> {
        if let Some(s) = self.fst.borrow().start() {
            let n = self.state_table.borrow().len();
            self.state_table.borrow_mut().push(Rc::new(
                RandState::new(s)
                    .with_nsamples(self.npath)
                    .with_length(0)
                    .with_select(0)
                    .with_parent(None),
            ));
            Ok(Some(n as StateId))
        } else {
            Ok(None)
        }
    }

    fn compute_trs_and_final_weight(&self, s: StateId) -> Result<(TrsVec<W>, Option<W>)> {
        if s == *self.superfinal.borrow() {
            let result = Ok((TrsVec::default(), Some(W::one())));
            return result;
        }
        let rstate = Rc::clone(self.state_table.borrow().get(s as usize).unwrap());
        self.sampler.borrow_mut().sample(&rstate)?;

        let aiter = self.fst.borrow().get_trs(rstate.state_id)?;
        let trs = aiter.trs();
        let num_trs = trs.len();

        let mut output_trs: Vec<Tr<W>> = vec![];
        let mut output_final_weight = None;

        for (&pos, &count) in self.sampler.borrow().iter() {
            let prob = (count as f32) / (rstate.nsamples as f32);
            if pos < num_trs {
                let tr = &trs[pos];
                let weight = if self.weighted {
                    W::new(-prob.ln())
                } else {
                    W::one()
                };
                output_trs.push(Tr::new(
                    tr.ilabel,
                    tr.olabel,
                    weight,
                    self.state_table.borrow().len() as StateId,
                ));
                let nrstate = RandState::new(tr.nextstate)
                    .with_nsamples(count)
                    .with_length(rstate.length + 1)
                    .with_select(pos)
                    .with_parent(Some(Rc::clone(&rstate)));
                self.state_table.borrow_mut().push(Rc::new(nrstate));
            } else {
                // Super-final transition.
                if self.weighted {
                    let weight = if self.remove_total_weight {
                        W::new(-prob.ln())
                    } else {
                        W::new(-(prob * self.npath as f32).ln())
                    };
                    output_final_weight = Some(weight);
                } else {
                    if *self.superfinal.borrow() == NO_STATE_ID {
                        *self.superfinal.borrow_mut() = self.state_table.borrow().len() as StateId;
                        self.state_table.borrow_mut().push(Rc::new(
                            RandState::new(NO_STATE_ID)
                                .with_nsamples(0)
                                .with_length(0)
                                .with_select(0)
                                .with_parent(None),
                        ));
                    }
                    for _ in 0..count {
                        output_trs.push(Tr::new(0, 0, W::one(), *self.superfinal.borrow()));
                    }
                }
            }
        }

        Ok((TrsVec(Arc::new(output_trs)), output_final_weight))
    }

    fn properties(&self) -> FstProperties {
        rand_gen_properties(self.fst.borrow().properties(), self.weighted)
            & FstProperties::copy_properties()
    }
}
