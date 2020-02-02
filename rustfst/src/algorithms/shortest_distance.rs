use std::collections::VecDeque;

use failure::Fallible;
use unsafe_unwrap::UnsafeUnwrap;

use crate::algorithms::reverse as reverse_f;
use crate::fst_impls::VectorFst;
use crate::fst_traits::{CoreFst, ExpandedFst};
use crate::semirings::{Semiring, SemiringProperties};
use crate::StateId;

/// This operation computes the shortest distance from the state `state_id` to every state.
/// The shortest distance from `p` to `q` is the ⊕-sum of the weights
/// of all the paths between `p` and `q`.
///
/// # Example
/// ```
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::fst_traits::MutableFst;
/// # use rustfst::algorithms::single_source_shortest_distance;
/// # use rustfst::Arc;
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
/// let dists = single_source_shortest_distance(&fst, s1).unwrap();
///
/// assert_eq!(dists, vec![
///     IntegerWeight::zero(),
///     IntegerWeight::one(),
///     IntegerWeight::new(55),
/// ]);
///
/// ```
pub fn single_source_shortest_distance<F: ExpandedFst>(
    fst: &F,
    state_id: StateId,
) -> Fallible<Vec<<F as CoreFst>::W>> {
    let mut d = vec![];
    let mut r = vec![];

    // Check whether the wFST contains the state
    if state_id < fst.num_states() {
        while d.len() <= state_id {
            d.push(F::W::zero());
            r.push(F::W::zero());
        }
        d[state_id] = F::W::one();
        r[state_id] = F::W::one();

        let mut queue = VecDeque::new();
        queue.push_back(state_id);

        while !queue.is_empty() {
            let state_cour = unsafe { queue.pop_front().unsafe_unwrap() };
            while d.len() <= state_cour {
                d.push(F::W::zero());
                r.push(F::W::zero());
            }
            let r2 = &r[state_cour].clone();
            r[state_cour] = F::W::zero();

            for arc in unsafe { fst.arcs_iter_unchecked(state_cour) } {
                let nextstate = arc.nextstate;
                while d.len() <= nextstate {
                    d.push(F::W::zero());
                    r.push(F::W::zero());
                }
                if d[nextstate] != d[nextstate].plus(&r2.times(&arc.weight)?)? {
                    d[nextstate] = d[nextstate].plus(&r2.times(&arc.weight)?)?;
                    r[nextstate] = r[nextstate].plus(&r2.times(&arc.weight)?)?;
                    if !queue.contains(&nextstate) {
                        queue.push_back(nextstate);
                    }
                }
            }
        }
    }

    Ok(d)
}

pub fn _shortest_distance<F: ExpandedFst>(fst: &F) -> Fallible<Vec<<F as CoreFst>::W>> {
    if !F::W::properties().contains(SemiringProperties::RIGHT_SEMIRING) {
        bail!("ShortestDistance: Weight needs to be right distributive");
    }
    if let Some(start_state) = fst.start() {
        return single_source_shortest_distance(fst, start_state);
    }
    Ok(vec![])
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
pub fn shortest_distance<F: ExpandedFst>(fst: &F, reverse: bool) -> Fallible<Vec<<F as CoreFst>::W>>
where
    <<F as CoreFst>::W as Semiring>::ReverseWeight: 'static,
{
    if !reverse {
        _shortest_distance(fst)
    } else {
        let rfst: VectorFst<_> = reverse_f(fst)?;
        let rdistance = _shortest_distance(&rfst)?;
        let mut distance = vec![];
        while distance.len() < (rdistance.len() - 1) {
            // TODO: Need to find a better to say that W::ReverseWeight::ReverseWeight == W
            let rw = rdistance[distance.len() + 1].reverse()?;
            distance.push(
                unsafe {
                    std::mem::transmute::<
                    &<<<F as CoreFst>::W as Semiring>::ReverseWeight as Semiring>::ReverseWeight,
                    &<F as CoreFst>::W,
                >(&rw)
                }
                .clone(),
            );
        }
        Ok(distance)
    }
}

pub mod revamp {
    use failure::Fallible;

    use crate::algorithms::arc_filters::{AnyArcFilter, ArcFilter};
    use crate::algorithms::queues::AutoQueue;
    use crate::algorithms::shortest_path::hack_convert_reverse_reverse;
    use crate::algorithms::Queue;
    use crate::fst_impls::VectorFst;
    use crate::fst_traits::{ExpandedFst, Fst, MutableFst};
    use crate::semirings::{Semiring, SemiringProperties};
    use crate::StateId;
    use failure::_core::marker::PhantomData;
    use nom::combinator::opt;

    pub struct ShortestDistanceConfig<W: Semiring, Q: Queue, A: ArcFilter<W>> {
        arc_filter: A,
        state_queue: Q,
        source: Option<StateId>,
        first_path: bool,
        // TODO: Shouldn't need that
        weight: PhantomData<W>,
    }

    impl<W: Semiring, Q: Queue, A: ArcFilter<W>> ShortestDistanceConfig<W, Q, A> {
        pub fn new(
            arc_filter: A,
            state_queue: Q,
            source: Option<StateId>,
            first_path: bool,
        ) -> Self {
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

    pub struct ShortestDistanceState<
        'a,
        W: Semiring,
        Q: Queue,
        A: ArcFilter<W>,
        F: ExpandedFst<W = W>,
    > {
        fst: &'a F,
        state_queue: Q,
        arc_filter: A,
        first_path: bool,
        enqueued: Vec<bool>,
        distance: Vec<W>,
        adder: Vec<W>,
        radder: Vec<W>,
        sources: Vec<Option<StateId>>,
        retain: bool,
        source_id: usize,
    }

    impl<'a, W: Semiring, Q: Queue, A: ArcFilter<W>, F: ExpandedFst<W = W>>
        ShortestDistanceState<'a, W, Q, A, F>
    {
        pub fn new(fst: &'a F, opts: ShortestDistanceConfig<W, Q, A>, retain: bool) -> Self {
            Self {
                fst,
                state_queue: opts.state_queue,
                arc_filter: opts.arc_filter,
                first_path: opts.first_path,
                distance: Vec::with_capacity(fst.num_states()),
                enqueued: Vec::with_capacity(fst.num_states()),
                adder: Vec::with_capacity(fst.num_states()),
                radder: Vec::with_capacity(fst.num_states()),
                sources: Vec::with_capacity(fst.num_states()),
                source_id: 0,
                retain,
            }
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

        pub fn shortest_distance(&mut self, source: Option<StateId>) -> Fallible<Vec<W>> {
            let start_state = self
                .fst
                .start()
                .ok_or_else(|| format_err!("Fst doesn't have s start state"))?;
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
            let source = source.unwrap_or(start_state);
            self.ensure_distance_index_is_valid(source);
            if self.retain {
                self.ensure_sources_index_is_valid(source);
                self.sources[source] = Some(self.source_id);
            }
            self.distance[source] = W::one();
            self.adder[source] = W::one();
            self.radder[source] = W::one();
            self.enqueued[source] = true;
            self.state_queue.enqueue(source);
            while !self.state_queue.is_empty() {
                let state = self.state_queue.head().unwrap();
                self.state_queue.dequeue();
                self.ensure_distance_index_is_valid(state);
                if self.first_path && self.fst.is_final(state)? {
                    break;
                }
                self.enqueued[state] = false;
                let r = self.radder[state].clone();
                self.radder[state] = W::zero();
                for arc in self.fst.arcs_iter(state)? {
                    let nextstate = arc.nextstate;
                    if !self.arc_filter.keep(arc) {
                        continue;
                    }
                    self.ensure_distance_index_is_valid(nextstate);
                    if self.retain {
                        self.ensure_sources_index_is_valid(nextstate);
                        if self.sources[nextstate] != Some(self.source_id) {
                            self.distance[nextstate] = W::zero();
                            self.adder[nextstate] = W::zero();
                            self.radder[nextstate] = W::zero();
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

    pub fn shortest_distance_with_config<W: Semiring, Q: Queue, A: ArcFilter<W>, F: MutableFst<W = W>>(
        fst: &F,
        opts: ShortestDistanceConfig<W, Q, A>,
    ) -> Fallible<Vec<W>> {
        let source = opts.source;
        let mut sd_state = ShortestDistanceState::new(fst, opts, false);
        sd_state.shortest_distance(source)
    }

    pub fn shortest_distance<F: MutableFst>(fst: &F, reverse: bool) -> Fallible<Vec<F::W>>
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
                distance.push(hack_convert_reverse_reverse(
                    rdistance[distance.len() + 1].reverse()?,
                ));
            }
            Ok(distance)
        }
    }

    /// Return the sum of the weight of all successful paths in an FST, i.e., the
    /// shortest-distance from the initial state to the final states..
    pub fn shortest_distance_3<F: MutableFst>(fst: &F) -> Fallible<F::W>
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
}
