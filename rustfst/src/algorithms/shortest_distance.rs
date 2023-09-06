use std::marker::PhantomData;

use anyhow::Result;

use crate::algorithms::queues::AutoQueue;
use crate::algorithms::tr_filters::{AnyTrFilter, TrFilter};
use crate::algorithms::Queue;
use crate::fst_impls::VectorFst;
use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::semirings::{ReverseBack, Semiring, SemiringProperties, WeightQuantize};
use crate::{StateId, Trs, KSHORTESTDELTA};
use std::borrow::Borrow;

pub(crate) struct ShortestDistanceInternalConfig<W: Semiring, Q: Queue, A: TrFilter<W>> {
    pub tr_filter: A,
    pub state_queue: Q,
    pub source: Option<StateId>,
    pub first_path: bool,
    pub delta: f32,
    // TODO: Shouldn't need that
    weight: PhantomData<W>,
}

impl<W: Semiring, Q: Queue, A: TrFilter<W>> ShortestDistanceInternalConfig<W, Q, A> {
    pub fn new(
        tr_filter: A,
        state_queue: Q,
        source: Option<StateId>,
        delta: f32,
        first_path: bool,
    ) -> Self {
        Self {
            tr_filter,
            state_queue,
            source,
            first_path,
            delta,
            weight: PhantomData,
        }
    }

    pub fn new_with_default(tr_filter: A, state_queue: Q, delta: f32) -> Self {
        Self::new(tr_filter, state_queue, None, delta, false)
    }
}

#[derive(Clone)]
pub(crate) struct ShortestDistanceState<W: Semiring, Q: Queue, A: TrFilter<W>> {
    state_queue: Q,
    tr_filter: A,
    first_path: bool,
    enqueued: Vec<bool>,
    distance: Vec<W>,
    adder: Vec<W>,
    radder: Vec<W>,
    sources: Vec<Option<StateId>>,
    retain: bool,
    source_id: usize,
    delta: f32,
}

impl<W: Semiring, Q: Queue, A: TrFilter<W>> std::fmt::Debug for ShortestDistanceState<W, Q, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ShortestDistanceState {{ ")?;
        write!(f, "state_queue : {:?}, ", self.state_queue)?;
        write!(f, "tr_filter : {:?}, ", self.tr_filter)?;
        write!(f, "first_path : {:?}, ", self.first_path)?;
        write!(f, "enqueued : {:?}, ", self.enqueued)?;
        write!(f, "distance : {:?}, ", self.distance)?;
        write!(f, "adder : {:?}, ", self.adder)?;
        write!(f, "radder : {:?}, ", self.radder)?;
        write!(f, "sources : {:?}, ", self.sources)?;
        write!(f, "retain : {:?}, ", self.retain)?;
        write!(f, "source_id : {:?} ", self.source_id)?;
        write!(f, "delta : {:?}", self.delta)?;
        write!(f, "}}")?;
        Ok(())
    }
}

macro_rules! ensure_distance_index_is_valid {
    ($s: ident, $index: expr) => {
        while $s.distance.len() <= $index {
            $s.distance.push(W::zero());
            $s.enqueued.push(false);
            $s.adder.push(W::zero());
            $s.radder.push(W::zero());
        }
    };
}

macro_rules! ensure_source_index_is_valid {
    ($s: ident, $index: expr) => {
        while $s.sources.len() <= $index {
            $s.sources.push(None);
        }
    };
}

impl<W: Semiring, Q: Queue, A: TrFilter<W>> ShortestDistanceState<W, Q, A> {
    pub fn new(
        fst_num_states: usize,
        state_queue: Q,
        tr_filter: A,
        first_path: bool,
        retain: bool,
        delta: f32,
    ) -> Self {
        Self {
            state_queue,
            tr_filter,
            first_path,
            distance: Vec::with_capacity(fst_num_states),
            enqueued: Vec::with_capacity(fst_num_states),
            adder: Vec::with_capacity(fst_num_states),
            radder: Vec::with_capacity(fst_num_states),
            sources: Vec::with_capacity(fst_num_states),
            source_id: 0,
            retain,
            delta,
        }
    }
    pub fn new_from_config(
        fst_num_states: usize,
        opts: ShortestDistanceInternalConfig<W, Q, A>,
        retain: bool,
    ) -> Self {
        Self::new(
            fst_num_states,
            opts.state_queue,
            opts.tr_filter,
            opts.first_path,
            retain,
            opts.delta,
        )
    }

    fn ensure_distance_index_is_valid(&mut self, index: usize) {
        while self.distance.len() <= index {
            self.distance.push(W::zero());
            self.enqueued.push(false);
            self.adder.push(W::zero());
            self.radder.push(W::zero());
        }
    }

    fn ensure_sources_index_is_valid(&mut self, index: usize) {
        while self.sources.len() <= index {
            self.sources.push(None);
        }
    }

    pub fn shortest_distance<F: ExpandedFst<W>, B: Borrow<F>>(
        &mut self,
        source: Option<StateId>,
        fst: B,
    ) -> Result<Vec<W>> {
        let start_state = match fst.borrow().start() {
            Some(start_state) => start_state,
            None => return Ok(vec![]),
        };
        let weight_properties = W::properties();
        if !weight_properties.contains(SemiringProperties::RIGHT_SEMIRING) {
            bail!("ShortestDistance: Weight needs to be right distributive")
        }
        if self.first_path && !weight_properties.contains(SemiringProperties::PATH) {
            bail!("ShortestDistance: The first_path option is disallowed when Weight does not have the path property")
        }
        self.state_queue.clear();
        if !self.retain {
            self.distance.clear();
            self.adder.clear();
            self.radder.clear();
            self.enqueued.clear();
        }

        let source = source.unwrap_or(start_state) as usize;
        self.ensure_distance_index_is_valid(source);
        if self.retain {
            self.ensure_sources_index_is_valid(source);
            self.sources[source] = Some(self.source_id as StateId);
        }
        self.distance[source] = W::one();
        self.adder[source] = W::one();
        self.radder[source] = W::one();
        self.enqueued[source] = true;
        self.state_queue.enqueue(source as StateId);
        while let Some(state) = self.state_queue.dequeue() {
            let state = state as usize;
            //            self.ensure_distance_index_is_valid(state);
            if self.first_path && fst.borrow().is_final(state as StateId)? {
                break;
            }
            self.enqueued[state] = false;
            let r = self.radder[state].clone();
            self.radder[state] = W::zero();
            for tr in fst.borrow().get_trs(state as StateId)?.trs() {
                let nextstate = tr.nextstate as usize;
                if !self.tr_filter.keep(tr) {
                    continue;
                }

                // Macros are used because the borrow checker is not smart enough to
                // understand than only some fields of the struct are modified.
                ensure_distance_index_is_valid!(self, nextstate);
                if self.retain {
                    ensure_source_index_is_valid!(self, nextstate);
                    if self.sources[nextstate] != Some(self.source_id as StateId) {
                        self.distance[nextstate] = W::zero();
                        self.adder[nextstate] = W::zero();
                        self.radder[nextstate] = W::zero();
                        self.enqueued[nextstate] = false;
                        self.sources[nextstate] = Some(self.source_id as StateId);
                    }
                }
                let nd = self.distance.get_mut(nextstate).unwrap();
                let na = self.adder.get_mut(nextstate).unwrap();
                let nr = self.radder.get_mut(nextstate).unwrap();
                let weight = r.times(&tr.weight)?;
                if !nd.approx_equal(nd.plus(&weight)?, self.delta) {
                    na.plus_assign(&weight)?;
                    *nd = na.clone();
                    nr.plus_assign(&weight)?;
                    if !self.enqueued[state] {
                        self.state_queue.enqueue(nextstate as StateId);
                        self.enqueued[nextstate] = true;
                    } else {
                        self.state_queue.update(nextstate as StateId);
                    }
                }
            }
        }
        self.source_id += 1;
        // TODO: This clone could be avoided
        Ok(self.distance.clone())
    }
}

pub(crate) fn shortest_distance_with_internal_config<
    W: Semiring,
    Q: Queue,
    A: TrFilter<W>,
    F: ExpandedFst<W>,
>(
    fst: &F,
    opts: ShortestDistanceInternalConfig<W, Q, A>,
) -> Result<Vec<W>> {
    let source = opts.source;
    let mut sd_state =
        ShortestDistanceState::<_, _, _>::new_from_config(fst.num_states(), opts, false);
    sd_state.shortest_distance::<F, _>(source, fst)
}

/// Configuration for shortest distance computation
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub struct ShortestDistanceConfig {
    delta: f32,
}

impl Default for ShortestDistanceConfig {
    fn default() -> Self {
        Self {
            delta: KSHORTESTDELTA,
        }
    }
}

impl ShortestDistanceConfig {
    pub fn new(delta: f32) -> Self {
        Self { delta }
    }
}

/// Compute the shortest distance from the initial state to every state.
/// The shortest distance from `p` to `q` is the ⊕-sum of the weights
/// of all the paths between `p` and `q`.
///
/// # Example
/// ```
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::fst_traits::MutableFst;
/// # use rustfst::algorithms::shortest_distance;
/// # use rustfst::Tr;
/// # use anyhow::Result;
/// fn main() -> Result<()> {
/// let mut fst = VectorFst::<IntegerWeight>::new();
/// let s0 = fst.add_state();
/// let s1 = fst.add_state();
/// let s2 = fst.add_state();
///
/// fst.set_start(s0).unwrap();
/// fst.add_tr(s0, Tr::new(32, 23, 18, s1));
/// fst.add_tr(s0, Tr::new(32, 23, 21, s2));
/// fst.add_tr(s1, Tr::new(32, 23, 55, s2));
///
/// let dists = shortest_distance(&fst, false)?;
///
/// assert_eq!(dists, vec![
///     IntegerWeight::one(),
///     IntegerWeight::new(18),
///     IntegerWeight::new(21 + 18*55),
/// ]);
/// # Ok(())
/// # }
/// ```
pub fn shortest_distance<W: Semiring, F: ExpandedFst<W>>(fst: &F, reverse: bool) -> Result<Vec<W>> {
    shortest_distance_with_config(fst, reverse, ShortestDistanceConfig::default())
}

/// Compute the shortest distance from the initial state to every
/// state, with configurable delta for comparison.
pub fn shortest_distance_with_config<W: Semiring, F: ExpandedFst<W>>(
    fst: &F,
    reverse: bool,
    config: ShortestDistanceConfig,
) -> Result<Vec<W>> {
    let delta = config.delta;
    let tr_filter = AnyTrFilter {};
    if !reverse {
        let queue = AutoQueue::new(fst, None, &tr_filter)?;
        let config = ShortestDistanceInternalConfig::new_with_default(tr_filter, queue, delta);
        shortest_distance_with_internal_config(fst, config)
    } else {
        let rfst: VectorFst<_> = crate::algorithms::reverse(fst)?;
        let state_queue = AutoQueue::new(&rfst, None, &tr_filter)?;
        let ropts = ShortestDistanceInternalConfig::new_with_default(tr_filter, state_queue, delta);
        let rdistance = shortest_distance_with_internal_config(&rfst, ropts)?;
        let mut distance = Vec::with_capacity(rdistance.len() - 1); //reversing added one state
        while distance.len() < rdistance.len() - 1 {
            distance.push(rdistance[distance.len() + 1].reverse_back()?);
        }
        Ok(distance)
    }
}

#[allow(unused)]
/// Return the sum of the weight of all successful paths in an FST, i.e., the
/// shortest-distance from the initial state to the final states..
fn shortest_distance_3<W: Semiring + WeightQuantize, F: MutableFst<W>>(
    fst: &F,
    delta: f32,
) -> Result<W>
where
    W::ReverseWeight: WeightQuantize,
{
    let weight_properties = W::properties();

    if weight_properties.contains(SemiringProperties::RIGHT_SEMIRING) {
        let distance =
            shortest_distance_with_config(fst, false, ShortestDistanceConfig::new(delta))?;
        let mut sum = W::zero();
        for (state, dist) in distance.iter().enumerate() {
            sum.plus_assign(
                dist.times(fst.final_weight(state as StateId)?.unwrap_or_else(W::zero))?,
            )?;
        }
        Ok(sum)
    } else {
        let distance =
            shortest_distance_with_config(fst, true, ShortestDistanceConfig::new(delta))?;
        if let Some(state) = fst.start() {
            let state = state as usize;
            if state < distance.len() {
                Ok(distance[state].clone())
            } else {
                Ok(W::zero())
            }
        } else {
            Ok(W::zero())
        }
    }
}
