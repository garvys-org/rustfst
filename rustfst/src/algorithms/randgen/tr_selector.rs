use crate::prelude::Fst;
use crate::{Semiring, StateId};
use anyhow::Result;
use rand::distributions::{Distribution, Uniform};
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Debug;

/// `TrSelector` implementors are used to select a random transition given an Fst
/// state `s`, returning a number `N` such that 0 <= `N` <= `fst.num_trs(s)`. If `N` is
/// `fst.num_trs(s)`, then the final weight is selected; otherwise the `N`-th transition is
/// selected. It is assumed these are not applied to any state which is neither
/// final nor has any arcs leaving it.
pub trait TrSelector: Debug {
    fn select_tr<W: Semiring, F: Fst<W>>(&mut self, fst: &F, state: StateId) -> Result<usize>;
}

/// Randomly selects a transition using the uniform distribution.
#[derive(Debug, Clone)]
pub struct UniformTrSelector {
    rng: ChaCha8Rng,
}

impl Default for UniformTrSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl UniformTrSelector {
    pub fn new() -> Self {
        Self {
            rng: ChaCha8Rng::from_entropy(),
        }
    }
    pub fn from_seed(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }
}

impl TrSelector for UniformTrSelector {
    fn select_tr<W: Semiring, F: Fst<W>>(&mut self, fst: &F, state: StateId) -> Result<usize> {
        let mut n = fst.num_trs(state)?;
        if fst.is_final(state)? {
            n += 1;
        }
        let uniform = Uniform::new_inclusive(0, n - 1);
        let res = uniform.sample(&mut self.rng);
        Ok(res)
    }
}
