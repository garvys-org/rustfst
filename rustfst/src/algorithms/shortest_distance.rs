use std::marker::PhantomData;

use failure::Fallible;

use crate::algorithms::arc_filters::{AnyArcFilter, ArcFilter};
use crate::algorithms::queues::AutoQueue;
use crate::algorithms::Queue;
use crate::fst_impls::VectorFst;
use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::semirings::{IntoSemiring, Semiring, SemiringProperties};
use crate::StateId;
use std::borrow::Borrow;

pub struct ShortestDistanceConfig<W: Semiring, Q: Queue, A: ArcFilter<W>> {
    pub arc_filter: A,
    pub state_queue: Q,
    pub source: Option<StateId>,
    pub first_path: bool,
    // TODO: Shouldn't need that
    weight: PhantomData<W>,
}

impl<W: Semiring, Q: Queue, A: ArcFilter<W>> ShortestDistanceConfig<W, Q, A> {
    pub fn new(arc_filter: A, state_queue: Q, source: Option<StateId>, first_path: bool) -> Self {
        Self {
            arc_filter,
            state_queue,
            source,
            first_path,
            weight: PhantomData,
        }
    }

    pub fn new_with_default(arc_filter: A, state_queue: Q) -> Self {
        Self::new(arc_filter, state_queue, None, false)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ShortestDistanceState<Q: Queue, F: ExpandedFst, B: Borrow<F>, A: ArcFilter<F::W>> {
    pub fst: B,
    state_queue: Q,
    arc_filter: A,
    first_path: bool,
    enqueued: Vec<bool>,
    distance: Vec<F::W>,
    adder: Vec<F::W>,
    radder: Vec<F::W>,
    sources: Vec<Option<StateId>>,
    retain: bool,
    source_id: usize,
}

impl<Q: Queue, F: ExpandedFst, B: Borrow<F>, A: ArcFilter<F::W>> std::fmt::Debug
    for ShortestDistanceState<Q, F, B, A>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ShortestDistanceState {{ ")?;
        write!(f, "fst : {:?}, ", self.fst.borrow())?;
        write!(f, "state_queue : {:?}, ", self.state_queue)?;
        write!(f, "arc_filter : {:?}, ", self.arc_filter)?;
        write!(f, "first_path : {:?}, ", self.first_path)?;
        write!(f, "enqueued : {:?}, ", self.enqueued)?;
        write!(f, "distance : {:?}, ", self.distance)?;
        write!(f, "adder : {:?}, ", self.adder)?;
        write!(f, "radder : {:?}, ", self.radder)?;
        write!(f, "sources : {:?}, ", self.sources)?;
        write!(f, "retain : {:?}, ", self.retain)?;
        write!(f, "source_id : {:?} ", self.source_id)?;
        write!(f, "}}")?;
        Ok(())
    }
}

macro_rules! ensure_distance_index_is_valid {
    ($s: ident, $index: expr) => {
        while $s.distance.len() <= $index {
            $s.distance.push(F::W::zero());
            $s.enqueued.push(false);
            $s.adder.push(F::W::zero());
            $s.radder.push(F::W::zero());
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

impl<Q: Queue, F: ExpandedFst, B: Borrow<F>, A: ArcFilter<F::W>> ShortestDistanceState<Q, F, B, A> {
    pub fn new(fst: B, state_queue: Q, arc_filter: A, first_path: bool, retain: bool) -> Self {
        Self {
            state_queue,
            arc_filter,
            first_path,
            distance: Vec::with_capacity(fst.borrow().num_states()),
            enqueued: Vec::with_capacity(fst.borrow().num_states()),
            adder: Vec::with_capacity(fst.borrow().num_states()),
            radder: Vec::with_capacity(fst.borrow().num_states()),
            sources: Vec::with_capacity(fst.borrow().num_states()),
            source_id: 0,
            retain,
            fst,
        }
    }
    pub fn new_from_config(fst: B, opts: ShortestDistanceConfig<F::W, Q, A>, retain: bool) -> Self {
        Self::new(
            fst,
            opts.state_queue,
            opts.arc_filter,
            opts.first_path,
            retain,
        )
    }

    fn ensure_distance_index_is_valid(&mut self, index: usize) {
        while self.distance.len() <= index {
            self.distance.push(F::W::zero());
            self.enqueued.push(false);
            self.adder.push(F::W::zero());
            self.radder.push(F::W::zero());
        }
    }

    fn ensure_sources_index_is_valid(&mut self, index: usize) {
        while self.sources.len() <= index {
            self.sources.push(None);
        }
    }

    pub fn shortest_distance(&mut self, source: Option<StateId>) -> Fallible<Vec<F::W>> {
        let start_state = match self.fst.borrow().start() {
            Some(start_state) => start_state,
            None => return Ok(vec![]),
        };
        let weight_properties = F::W::properties();
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

        let source = source.unwrap_or(start_state);
        self.ensure_distance_index_is_valid(source);
        if self.retain {
            self.ensure_sources_index_is_valid(source);
            self.sources[source] = Some(self.source_id);
        }
        self.distance[source] = F::W::one();
        self.adder[source] = F::W::one();
        self.radder[source] = F::W::one();
        self.enqueued[source] = true;
        self.state_queue.enqueue(source);
        while !self.state_queue.is_empty() {
            let state = self.state_queue.head().unwrap();
            self.state_queue.dequeue();
            //            self.ensure_distance_index_is_valid(state);
            if self.first_path && self.fst.borrow().is_final(state)? {
                break;
            }
            self.enqueued[state] = false;
            let r = self.radder[state].clone();
            self.radder[state] = F::W::zero();
            for arc in self.fst.borrow().arcs_iter(state)? {
                let nextstate = arc.nextstate;
                if !self.arc_filter.keep(arc) {
                    continue;
                }

                // Macros are used because the borrow checker is not smart enough to
                // understand than only some fields of the struct are modified.
                ensure_distance_index_is_valid!(self, nextstate);
                if self.retain {
                    ensure_source_index_is_valid!(self, nextstate);
                    if self.sources[nextstate] != Some(self.source_id) {
                        self.distance[nextstate] = F::W::zero();
                        self.adder[nextstate] = F::W::zero();
                        self.radder[nextstate] = F::W::zero();
                        self.enqueued[nextstate] = false;
                        self.sources[nextstate] = Some(self.source_id);
                    }
                }
                let nd = self.distance.get_mut(nextstate).unwrap();
                let na = self.adder.get_mut(nextstate).unwrap();
                let nr = self.radder.get_mut(nextstate).unwrap();
                let weight = r.times(&arc.weight)?;
                if *nd != nd.plus(&weight)? {
                    na.plus_assign(&weight)?;
                    *nd = na.clone();
                    nr.plus_assign(&weight)?;
                    if !self.enqueued[state] {
                        self.state_queue.enqueue(nextstate);
                        self.enqueued[nextstate] = true;
                    } else {
                        self.state_queue.update(nextstate);
                    }
                }
            }
        }
        self.source_id += 1;
        // TODO: This clone could be avoided
        Ok(self.distance.clone())
    }
}

pub fn shortest_distance_with_config<
    W: Semiring,
    Q: Queue,
    A: ArcFilter<W>,
    F: ExpandedFst<W = W>,
>(
    fst: &F,
    opts: ShortestDistanceConfig<W, Q, A>,
) -> Fallible<Vec<W>> {
    let source = opts.source;
    let mut sd_state = ShortestDistanceState::<_, F, _, _>::new_from_config(fst, opts, false);
    sd_state.shortest_distance(source)
}

/// This operation computes the shortest distance from the initial state to every state.
/// The shortest distance from `p` to `q` is the ⊕-sum of the weights
/// of all the paths between `p` and `q`.
///
/// # Example
/// ```
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::fst_traits::MutableFst;
/// # use rustfst::algorithms::shortest_distance;
/// # use rustfst::Arc;
/// # use failure::Fallible;
/// fn main() -> Fallible<()> {
/// let mut fst = VectorFst::<IntegerWeight>::new();
/// let s0 = fst.add_state();
/// let s1 = fst.add_state();
/// let s2 = fst.add_state();
///
/// fst.set_start(s0).unwrap();
/// fst.add_arc(s0, Arc::new(32, 23, 18, s1));
/// fst.add_arc(s0, Arc::new(32, 23, 21, s2));
/// fst.add_arc(s1, Arc::new(32, 23, 55, s2));
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
pub fn shortest_distance<F: ExpandedFst>(fst: &F, reverse: bool) -> Fallible<Vec<F::W>>
where
    F::W: 'static,
{
    if !reverse {
        let arc_filter = AnyArcFilter {};
        let queue = AutoQueue::new(fst, None, &arc_filter)?;
        let config = ShortestDistanceConfig::new_with_default(arc_filter, queue);
        shortest_distance_with_config(fst, config)
    } else {
        let arc_filter = AnyArcFilter {};
        let rfst: VectorFst<_> = crate::algorithms::reverse(fst)?;
        let state_queue = AutoQueue::new(&rfst, None, &arc_filter)?;
        let ropts = ShortestDistanceConfig::new_with_default(arc_filter, state_queue);
        let rdistance = shortest_distance_with_config(&rfst, ropts)?;
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
fn shortest_distance_3<F: MutableFst>(fst: &F) -> Fallible<F::W>
where
    F::W: 'static,
{
    let weight_properties = F::W::properties();

    if weight_properties.contains(SemiringProperties::RIGHT_SEMIRING) {
        let distance = shortest_distance(fst, false)?;
        let mut sum = F::W::zero();
        let zero = F::W::zero();
        for state in 0..distance.len() {
            sum.plus_assign(distance[state].times(fst.final_weight(state)?.unwrap_or(&zero))?)?;
        }
        Ok(sum)
    } else {
        let distance = shortest_distance(fst, true)?;
        if let Some(state) = fst.start() {
            if state < distance.len() {
                Ok(distance[state].clone())
            } else {
                Ok(F::W::zero())
            }
        } else {
            Ok(F::W::zero())
        }
    }
}
