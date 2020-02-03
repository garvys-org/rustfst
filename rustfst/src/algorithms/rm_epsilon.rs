use std::cell::UnsafeCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;
use std::slice::Iter as IterSlice;

use failure::Fallible;
use unsafe_unwrap::UnsafeUnwrap;

use crate::{Arc, EPS_LABEL, Label, StateId, SymbolTable};
use crate::algorithms::arc_filters::{ArcFilter, EpsilonArcFilter};
use crate::algorithms::cache::{CacheImpl, FstImpl};
use crate::algorithms::dfs_visit::dfs_visit;
use crate::algorithms::dynamic_fst::StatesIteratorDynamicFst;
use crate::algorithms::Queue;
use crate::algorithms::queues::AutoQueue;
use crate::algorithms::shortest_distance::{ShortestDistanceConfig, ShortestDistanceState};
use crate::algorithms::top_sort::TopOrderVisitor;
use crate::algorithms::visitors::SccVisitor;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{MutableFst, StateIterator};
use crate::fst_traits::{ArcIterator, CoreFst, Fst};
use crate::semirings::Semiring;

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

    let mut rmeps_state = RmEpsilonState::new(fst, opts);
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

#[derive(Hash, Debug, PartialOrd, PartialEq, Eq)]
struct Element {
    ilabel: Label,
    olabel: Label,
    nextstate: StateId,
}

struct RmEpsilonState<'a, F: MutableFst, Q: Queue> {
    fst: &'a F,
    visited: Vec<bool>,
    visited_states: Vec<StateId>,
    element_map: HashMap<Element, (StateId, usize)>,
    expand_id: usize,
    sd_state: ShortestDistanceState<'a, F::W, Q, EpsilonArcFilter, F>,
}

impl<'a, F: MutableFst, Q: Queue> RmEpsilonState<'a, F, Q>
where
    <<F as CoreFst>::W as Semiring>::ReverseWeight: 'static,
{
    pub fn new(fst: &'a F, opts: RmEpsilonConfig<F::W, Q>) -> Self {
        Self {
            fst,
            visited: vec![],
            visited_states: vec![],
            element_map: HashMap::new(),
            expand_id: 0,
            sd_state: ShortestDistanceState::new_from_config(fst, opts.sd_opts, true),
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
            for arc in self.fst.arcs_iter(state)? {
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
                distance[state].times(self.fst.final_weight(state)?.unwrap_or(&zero))?,
            )?;
        }

        while let Some(s) = self.visited_states.pop() {
            self.visited[s] = false;
        }

        self.expand_id += 1;

        Ok((arcs, final_weight))
    }
}

pub(crate) struct RmEpsilonImpl<'a, F: MutableFst, Q: Queue> {
    rmeps_state: RmEpsilonState<'a, F, Q>,
    cache_impl: CacheImpl<F::W>,
}

impl<'a, F: MutableFst, Q: Queue> FstImpl for RmEpsilonImpl<'a, F, Q>
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
        Ok(self.rmeps_state.fst.start())
    }

    fn compute_final(&mut self, state: usize) -> Fallible<Option<Self::W>> {
        // A bit hacky as the final weight is computed inside the expand function.
        // Should in theory never be called
        self.expand(state)?;
        let weight = self.cache_impl.final_weight(state)?;
        Ok(weight.cloned())
    }
}

pub struct RmEpsilonFst<'a, F: MutableFst, Q: Queue> {
    pub(crate) fst_impl: UnsafeCell<RmEpsilonImpl<'a, F, Q>>,
    pub(crate) isymt: Option<Rc<SymbolTable>>,
    pub(crate) osymt: Option<Rc<SymbolTable>>,
}

impl<'a, F: MutableFst, Q: Queue> RmEpsilonFst<'a, F, Q>
where
    F::W: 'static,
{
    fn num_known_states(&self) -> usize {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_ref().unwrap() };
        fst_impl.num_known_states()
    }
}

impl<'a, F: MutableFst, Q: Queue> PartialEq for RmEpsilonFst<'a, F, Q> {
    fn eq(&self, other: &Self) -> bool {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_ref().unwrap() };

        let ptr_other = other.fst_impl.get();
        let fst_impl_other = unsafe { ptr_other.as_ref().unwrap() };

        fst_impl.eq(fst_impl_other)
    }
}

impl<'a, F: MutableFst, Q: Queue> std::fmt::Debug for RmEpsilonFst<'a, F, Q>
where
    F::W: 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_ref().unwrap() };
        write!(f, "RmEpsilonFst {{ {:?} }}", &fst_impl)
    }
}

impl<'a, F: MutableFst, Q: Queue> Clone for RmEpsilonFst<'a, F, Q>
where
    F::W: 'static,
{
    fn clone(&self) -> Self {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_ref().unwrap() };
        Self {
            fst_impl: UnsafeCell::new(fst_impl.clone()),
            isymt: self.input_symbols(),
            osymt: self.output_symbols(),
        }
    }
}

impl<'a, F: MutableFst, Q: Queue> CoreFst for RmEpsilonFst<'a, F, Q>
where
    F::W: 'static,
{
    type W = F::W;

    fn start(&self) -> Option<usize> {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_mut().unwrap() };
        fst_impl.start().unwrap()
    }

    fn final_weight(&self, state_id: usize) -> Fallible<Option<&Self::W>> {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_mut().unwrap() };
        fst_impl.final_weight(state_id)
    }

    unsafe fn final_weight_unchecked(&self, state_id: usize) -> Option<&Self::W> {
        self.final_weight(state_id).unwrap()
    }

    fn num_arcs(&self, s: usize) -> Fallible<usize> {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_mut().unwrap() };
        fst_impl.num_arcs(s)
    }

    unsafe fn num_arcs_unchecked(&self, s: usize) -> usize {
        self.num_arcs(s).unwrap()
    }
}

impl<'a, 'b, F: MutableFst, Q: Queue> ArcIterator<'a> for RmEpsilonFst<'b, F, Q>
where
    F::W: 'static,
{
    type Iter = IterSlice<'a, Arc<F::W>>;

    fn arcs_iter(&'a self, state_id: usize) -> Fallible<Self::Iter> {
        let ptr = self.fst_impl.get();
        let fst_impl = unsafe { ptr.as_mut().unwrap() };
        fst_impl.arcs_iter(state_id)
    }

    unsafe fn arcs_iter_unchecked(&'a self, state_id: usize) -> Self::Iter {
        self.arcs_iter(state_id).unwrap()
    }
}

impl<'a, 'b, F: MutableFst, Q: Queue> Iterator
    for StatesIteratorDynamicFst<'a, RmEpsilonFst<'b, F, Q>>
where
    F::W: 'static,
{
    type Item = StateId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.s < self.fst.num_known_states() {
            let s_cur = self.s;
            // Force expansion of the state
            self.fst.arcs_iter(s_cur).unwrap();
            self.s += 1;
            Some(s_cur)
        } else {
            None
        }
    }
}

impl<'a, 'b, F: MutableFst, Q: Queue> StateIterator<'a> for RmEpsilonFst<'b, F, Q>
where
    F::W: 'static,
    'a: 'b
{
    type Iter = StatesIteratorDynamicFst<'a, RmEpsilonFst<'b, F, Q>>;

    fn states_iter(&'a self) -> Self::Iter {
        self.start();
        StatesIteratorDynamicFst { fst: &self, s: 0 }
    }
}

impl<'a, F: MutableFst, Q: Queue> Fst for RmEpsilonFst<'a, F, Q>
where
    F::W: 'static,
{
    fn input_symbols(&self) -> Option<Rc<SymbolTable>> {
        self.isymt.clone()
    }

    fn output_symbols(&self) -> Option<Rc<SymbolTable>> {
        self.osymt.clone()
    }
}
