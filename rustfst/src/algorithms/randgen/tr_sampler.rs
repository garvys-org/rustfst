use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

use anyhow::Result;

use crate::algorithms::randgen::rand_state::RandState;
use crate::algorithms::randgen::TrSelector;
use crate::prelude::Fst;
use crate::Semiring;

/// This class, given a TrSelector, samples, with replacement, multiple random
/// transitions from an FST's state. This is a generic version with a
/// straightforward use of the tr selector. Specializations may be defined for
/// tr selectors for greater efficiency or special behavior.
pub struct TrSampler<W: Semiring, F: Fst<W>, B: Borrow<F>, S: TrSelector> {
    max_length: usize,
    selector: S,
    fst: B,
    sample_map: BTreeMap<usize, usize>,
    ghost: PhantomData<(W, F)>,
}

impl<W, F, B, S> Debug for TrSampler<W, F, B, S>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F>,
    S: TrSelector,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TrSampler {{ max_length : {:?}, selector : {:?}, fst : {:?}, sample_map : {:?} }}",
            self.max_length,
            self.selector,
            self.fst.borrow(),
            self.sample_map
        )
    }
}

impl<W, F, B, S> TrSampler<W, F, B, S>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F>,
    S: TrSelector,
{
    pub fn new(fst: B, selector: S, max_length: usize) -> Self {
        Self {
            fst,
            selector,
            max_length,
            sample_map: BTreeMap::new(),
            ghost: PhantomData,
        }
    }

    /// Samples a fixed number of samples from the given state. The length argument
    /// specifies the length of the path to the state. Returns true if the samples
    /// were collected. No samples may be collected if either there are no
    /// transitions leaving the state and the state is non-final, or if the path
    /// length has been exceeded. Iterator members are provided to read the samples
    /// in the order in which they were collected.
    pub fn sample(&mut self, rstate: &RandState) -> Result<bool> {
        self.sample_map.clear();
        if (self.fst.borrow().num_trs(rstate.state_id)? == 0
            && !self.fst.borrow().is_final(rstate.state_id)?)
            || rstate.length == self.max_length
        {
            // self.reset();
            return Ok(false);
        }
        for _ in 0..rstate.nsamples {
            let selected = self
                .selector
                .select_tr(self.fst.borrow(), rstate.state_id)?;
            *self.sample_map.entry(selected).or_insert(0) += 1;
        }
        // self.reset();
        Ok(true)
    }

    pub fn iter(&self) -> std::collections::btree_map::Iter<usize, usize> {
        self.sample_map.iter()
    }
}
