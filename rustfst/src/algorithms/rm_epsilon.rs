use std::cell::UnsafeCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::slice::Iter as IterSlice;

use failure::Fallible;
use unsafe_unwrap::UnsafeUnwrap;

use crate::algorithms::arc_filters::{ArcFilter, EpsilonArcFilter};
use crate::algorithms::cache::{CacheImpl, FstImpl};
use crate::algorithms::dfs_visit::dfs_visit;
use crate::algorithms::queues::{AutoQueue, FifoQueue};
use crate::algorithms::shortest_distance::{ShortestDistanceConfig, ShortestDistanceState};
use crate::algorithms::top_sort::TopOrderVisitor;
use crate::algorithms::visitors::SccVisitor;
use crate::algorithms::{BorrowFst, Queue};
use crate::fst_properties::FstProperties;
use crate::fst_traits::{ArcIterator, CoreFst, Fst};
use crate::fst_traits::{MutableFst, StateIterator};
use crate::semirings::Semiring;
use crate::{Arc, Label, StateId, SymbolTable, EPS_LABEL};

pub struct RmEpsilonConfig<W: Semiring, Q: Queue> {
    sd_opts: ShortestDistanceConfig<W, Q, EpsilonArcFilter>,
    connect: bool,
    weight_threshold: W,
    state_threshold: Option<StateId>,
}

impl<W: Semiring, Q: Queue> RmEpsilonConfig<W, Q> {
    pub fn new(
        queue: Q,
        connect: bool,
        weight_threshold: W,
        state_threshold: Option<StateId>,
    ) -> Self {
        Self {
            sd_opts: ShortestDistanceConfig::new_with_default(EpsilonArcFilter {}, queue),
            connect,
            weight_threshold,
            state_threshold,
        }
    }

    pub fn new_with_default(queue: Q) -> Self {
        Self::new(queue, true, W::zero(), None)
    }
}

/// This operation removes epsilon-transitions (when both the input and
/// output labels are an epsilon) from a transducer. The result will be an
/// equivalent FST that has no such epsilon transitions.
///
/// # Example
/// ```
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::fst_traits::MutableFst;
/// # use rustfst::algorithms::rm_epsilon;
/// # use rustfst::Arc;
/// # use rustfst::EPS_LABEL;
/// # use failure::Fallible;
/// # fn main() -> Fallible<()> {
/// let mut fst = VectorFst::new();
/// let s0 = fst.add_state();
/// let s1 = fst.add_state();
/// fst.add_arc(s0, Arc::new(32, 25, IntegerWeight::new(78), s1));
/// fst.add_arc(s1, Arc::new(EPS_LABEL, EPS_LABEL, IntegerWeight::new(13), s0));
/// fst.set_start(s0)?;
/// fst.set_final(s0, IntegerWeight::new(5))?;
///
/// let mut fst_no_epsilon = fst.clone();
/// rm_epsilon(&mut fst_no_epsilon)?;
///
/// let mut fst_no_epsilon_ref = VectorFst::<IntegerWeight>::new();
/// let s0 = fst_no_epsilon_ref.add_state();
/// let s1 = fst_no_epsilon_ref.add_state();
/// fst_no_epsilon_ref.add_arc(s0, Arc::new(32, 25, 78, s1));
/// fst_no_epsilon_ref.add_arc(s1, Arc::new(32, 25, 78 * 13, s1));
/// fst_no_epsilon_ref.set_start(s0)?;
/// fst_no_epsilon_ref.set_final(s0, 5)?;
/// fst_no_epsilon_ref.set_final(s1, 5 * 13)?;
///
/// assert_eq!(fst_no_epsilon, fst_no_epsilon_ref);
/// # Ok(())
/// # }
/// ```
pub fn rm_epsilon<F: MutableFst>(fst: &mut F) -> Fallible<()>
where
    F::W: 'static,
{
    let arc_filter = EpsilonArcFilter {};
    let queue = AutoQueue::new(fst, None, &arc_filter)?;
    let opts = RmEpsilonConfig::new_with_default(queue);
    rm_epsilon_with_config(fst, opts)
}
pub fn rm_epsilon_with_config<F: MutableFst, Q: Queue>(
    fst: &mut F,
    opts: RmEpsilonConfig<F::W, Q>,
) -> Fallible<()>
where
    <<F as CoreFst>::W as Semiring>::ReverseWeight: 'static,
    F::W: 'static,
{
    let connect = opts.connect;
    let weight_threshold = opts.weight_threshold.clone();
    let state_threshold = opts.state_threshold.clone();

    let start_state = fst.start();
    if start_state.is_none() {
        return Ok(());
    }
    let start_state = unsafe { start_state.unsafe_unwrap() };

    // noneps_in[s] will be set to true iff s admits a non-epsilon incoming
    // transition or is the start state.
    let mut noneps_in = vec![false; fst.num_states()];
    noneps_in[start_state] = true;

    for state in 0..fst.num_states() {
        for arc in fst.arcs_iter(state)? {
            if arc.ilabel != EPS_LABEL || arc.olabel != EPS_LABEL {
                noneps_in[arc.nextstate] = true;
            }
        }
    }

    // States sorted in topological order when (acyclic) or generic topological
    // order (cyclic).
    let mut states = vec![];

    let fst_props = fst.properties()?;

    if fst_props.contains(FstProperties::TOP_SORTED) {
        states = (0..fst.num_states()).collect();
    } else if fst_props.contains(FstProperties::TOP_SORTED) {
        let mut visitor = TopOrderVisitor::new();
        dfs_visit(fst, &mut visitor, &EpsilonArcFilter {}, false);

        for i in 0..visitor.order.len() {
            states[visitor.order[i]] = i;
        }
    } else {
        let mut visitor = SccVisitor::new(fst, true, false);
        dfs_visit(fst, &mut visitor, &EpsilonArcFilter {}, false);

        let scc = visitor.scc.as_ref().unwrap();

        let mut first = vec![None; scc.len()];
        let mut next = vec![None; scc.len()];

        for i in 0..scc.len() {
            if first[scc[i] as usize].is_some() {
                next[i] = first[scc[i] as usize];
            }
            first[scc[i] as usize] = Some(i);
        }

        for i in 0..first.len() {
            let mut opt_j = first[i];
            while let Some(j) = opt_j {
                states.push(j);
                opt_j = next[j];
            }
        }
    }

    let mut rmeps_state = RmEpsilonState::new(&*fst, opts);
    let zero = F::W::zero();

    let mut v = Vec::with_capacity(states.len());
    for state in states.into_iter().rev() {
        if !noneps_in[state] {
            continue;
        }
        let (arcs, final_weight) = rmeps_state.expand(state)?;

        // Copy everything, not great ...
        v.push((state, (arcs, final_weight)));
    }

    for (state, (arcs, final_weight)) in v.into_iter() {
        unsafe {
            // TODO: Use these arcs instead of cloning
            fst.pop_arcs_unchecked(state);
            fst.set_arcs_unchecked(state, arcs.into_iter().rev().collect());
            if final_weight != zero {
                fst.set_final_unchecked(state, final_weight);
            } else {
                fst.delete_final_weight_unchecked(state);
            }
        }
    }

    if connect || weight_threshold != F::W::zero() || state_threshold != None {
        for s in 0..fst.num_states() {
            if !noneps_in[s] {
                fst.delete_arcs(s)?;
            }
        }
    }

    if weight_threshold != F::W::zero() || state_threshold != None {
        todo!("Implement Prune!")
    }

    if connect && weight_threshold == F::W::zero() && state_threshold == None {
        crate::algorithms::connect(fst)?;
    }
    Ok(())
}

#[derive(Hash, Debug, PartialOrd, PartialEq, Eq, Clone)]
struct Element {
    ilabel: Label,
    olabel: Label,
    nextstate: StateId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RmEpsilonState<F: MutableFst, B: BorrowFst<F>, Q: Queue> {
    visited: Vec<bool>,
    visited_states: Vec<StateId>,
    element_map: HashMap<Element, (StateId, usize)>,
    expand_id: usize,
    sd_state: ShortestDistanceState<Q, F, B, EpsilonArcFilter>,
}

impl<F: MutableFst, B: BorrowFst<F>, Q: Queue> RmEpsilonState<F, B, Q>
where
    <<F as CoreFst>::W as Semiring>::ReverseWeight: 'static,
{
    pub fn new(fst: B, opts: RmEpsilonConfig<F::W, Q>) -> Self {
        Self {
            sd_state: ShortestDistanceState::new_from_config(fst, opts.sd_opts, true),
            visited: vec![],
            visited_states: vec![],
            element_map: HashMap::new(),
            expand_id: 0,
        }
    }

    pub fn expand(&mut self, source: StateId) -> Fallible<(Vec<Arc<F::W>>, F::W)> {
        let zero = F::W::zero();
        let distance = self.sd_state.shortest_distance(Some(source))?;

        let arc_filter = EpsilonArcFilter {};

        let mut eps_queue = vec![source];

        let mut arcs = vec![];
        let mut final_weight = F::W::zero();
        while let Some(state) = eps_queue.pop() {
            while self.visited.len() <= state {
                self.visited.push(false);
            }
            if self.visited[state] {
                continue;
            }
            self.visited[state] = true;
            self.visited_states.push(state);
            for arc in self.sd_state.fst.borrow().arcs_iter(state)? {
                // TODO: Remove this clone
                let mut arc = arc.clone();
                arc.weight = distance[state].times(&arc.weight)?;
                if arc_filter.keep(&arc) {
                    while self.visited.len() <= state {
                        self.visited.push(false);
                    }
                    if !self.visited[arc.nextstate] {
                        eps_queue.push(arc.nextstate);
                    }
                } else {
                    let elt = Element {
                        ilabel: arc.ilabel,
                        olabel: arc.olabel,
                        nextstate: arc.nextstate,
                    };
                    let val = (self.expand_id, arcs.len());

                    match self.element_map.entry(elt) {
                        Entry::Vacant(e) => {
                            e.insert(val);
                            arcs.push(arc);
                        }
                        Entry::Occupied(mut e) => {
                            if e.get().0 == self.expand_id {
                                unsafe {
                                    arcs.get_unchecked_mut(e.get().1)
                                        .weight
                                        .plus_assign(&arc.weight)?;
                                }
                            } else {
                                e.get_mut().0 = self.expand_id;
                                e.get_mut().1 = arcs.len();
                                arcs.push(arc);
                            }
                        }
                    };
                }
            }
            final_weight.plus_assign(
                distance[state].times(
                    self.sd_state
                        .fst
                        .borrow()
                        .final_weight(state)?
                        .unwrap_or(&zero),
                )?,
            )?;
        }

        while let Some(s) = self.visited_states.pop() {
            self.visited[s] = false;
        }

        self.expand_id += 1;

        Ok((arcs, final_weight))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RmEpsilonImpl<F: MutableFst, B: BorrowFst<F>> {
    rmeps_state: RmEpsilonState<F, B, FifoQueue>,
    cache_impl: CacheImpl<F::W>,
}

impl<F: MutableFst, B: BorrowFst<F>> RmEpsilonImpl<F, B>
where
    <<F as CoreFst>::W as Semiring>::ReverseWeight: 'static,
{
    fn new(fst: B) -> Self {
        Self {
            cache_impl: CacheImpl::new(),
            rmeps_state: RmEpsilonState::new(
                fst,
                RmEpsilonConfig::new_with_default(FifoQueue::default()),
            ),
        }
    }
}

impl<F: MutableFst, B: BorrowFst<F>> FstImpl for RmEpsilonImpl<F, B>
where
    F::W: 'static,
{
    type W = F::W;

    fn cache_impl_mut(&mut self) -> &mut CacheImpl<Self::W> {
        &mut self.cache_impl
    }

    fn cache_impl_ref(&self) -> &CacheImpl<Self::W> {
        &self.cache_impl
    }

    fn expand(&mut self, state: usize) -> Fallible<()> {
        let (arcs, final_weight) = self.rmeps_state.expand(state)?;
        let zero = F::W::zero();

        for arc in arcs.into_iter().rev() {
            self.cache_impl.push_arc(state, arc)?;
        }
        if final_weight != zero {
            self.cache_impl
                .set_final_weight(state, Some(final_weight))?;
        } else {
            self.cache_impl.set_final_weight(state, None)?;
        }

        Ok(())
    }

    fn compute_start(&mut self) -> Fallible<Option<usize>> {
        Ok(self.rmeps_state.sd_state.fst.borrow().start())
    }

    fn compute_final(&mut self, state: usize) -> Fallible<Option<Self::W>> {
        // A bit hacky as the final weight is computed inside the expand function.
        // Should in theory never be called
        self.expand(state)?;
        let weight = self.cache_impl.final_weight(state)?;
        Ok(weight.cloned())
    }
}

pub struct RmEpsilonFst<F: MutableFst, B: BorrowFst<F>> {
    pub(crate) fst_impl: UnsafeCell<RmEpsilonImpl<F, B>>,
    pub(crate) isymt: Option<Rc<SymbolTable>>,
    pub(crate) osymt: Option<Rc<SymbolTable>>,
}

impl<F: MutableFst, B: BorrowFst<F>> RmEpsilonFst<F, B>
where
    <<F as CoreFst>::W as Semiring>::ReverseWeight: 'static,
{
    pub fn new(fst: B) -> Self {
        let isymt = fst.borrow().input_symbols();
        let osymt = fst.borrow().output_symbols();
        Self {
            fst_impl: UnsafeCell::new(RmEpsilonImpl::new(fst)),
            isymt,
            osymt,
        }
    }
}

dynamic_fst!("RmEpsilonFst", RmEpsilonFst<F, B>, [F => MutableFst] [B => BorrowFst<F>]);
