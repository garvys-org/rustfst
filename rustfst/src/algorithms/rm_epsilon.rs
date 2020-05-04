use std::borrow::Borrow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use anyhow::Result;
use unsafe_unwrap::UnsafeUnwrap;

use crate::algorithms::cache::{CacheImpl, FstImpl};
use crate::algorithms::dfs_visit::dfs_visit;
use crate::algorithms::lazy_fst::LazyFst;
use crate::algorithms::queues::{AutoQueue, FifoQueue};
use crate::algorithms::shortest_distance::{ShortestDistanceConfig, ShortestDistanceState};
use crate::algorithms::top_sort::TopOrderVisitor;
use crate::algorithms::tr_filters::{EpsilonTrFilter, TrFilter};
use crate::algorithms::visitors::SccVisitor;
use crate::algorithms::Queue;
use crate::fst_properties::FstProperties;
use crate::fst_traits::CoreFst;
use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::{Label, StateId, Tr, EPS_LABEL};

pub struct RmEpsilonConfig<W: Semiring, Q: Queue> {
    sd_opts: ShortestDistanceConfig<W, Q, EpsilonTrFilter>,
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
            sd_opts: ShortestDistanceConfig::new_with_default(EpsilonTrFilter {}, queue),
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
/// # Example 1
/// ```
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::fst_traits::MutableFst;
/// # use rustfst::algorithms::rm_epsilon;
/// # use rustfst::Tr;
/// # use rustfst::EPS_LABEL;
/// # use anyhow::Result;
/// # fn main() -> Result<()> {
/// let mut fst = VectorFst::new();
/// let s0 = fst.add_state();
/// let s1 = fst.add_state();
/// fst.add_tr(s0, Tr::new(32, 25, IntegerWeight::new(78), s1));
/// fst.add_tr(s1, Tr::new(EPS_LABEL, EPS_LABEL, IntegerWeight::new(13), s0));
/// fst.set_start(s0)?;
/// fst.set_final(s0, IntegerWeight::new(5))?;
///
/// let mut fst_no_epsilon = fst.clone();
/// rm_epsilon(&mut fst_no_epsilon)?;
///
/// let mut fst_no_epsilon_ref = VectorFst::<IntegerWeight>::new();
/// let s0 = fst_no_epsilon_ref.add_state();
/// let s1 = fst_no_epsilon_ref.add_state();
/// fst_no_epsilon_ref.add_tr(s0, Tr::new(32, 25, 78, s1));
/// fst_no_epsilon_ref.add_tr(s1, Tr::new(32, 25, 78 * 13, s1));
/// fst_no_epsilon_ref.set_start(s0)?;
/// fst_no_epsilon_ref.set_final(s0, 5)?;
/// fst_no_epsilon_ref.set_final(s1, 5 * 13)?;
///
/// assert_eq!(fst_no_epsilon, fst_no_epsilon_ref);
/// # Ok(())
/// # }
/// ```
///
/// # Example 2
///
/// ## Input
///
/// ![rmepsilon_in](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/rmepsilon_in.svg?sanitize=true)
///
/// ## RmEpsilon
///
/// ![rmepsilon_out](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/rmepsilon_out.svg?sanitize=true)
///
pub fn rm_epsilon<F: MutableFst>(fst: &mut F) -> Result<()>
where
    F::W: 'static,
{
    let tr_filter = EpsilonTrFilter {};
    let queue = AutoQueue::new(fst, None, &tr_filter)?;
    let opts = RmEpsilonConfig::new_with_default(queue);
    rm_epsilon_with_config(fst, opts)
}
pub fn rm_epsilon_with_config<F: MutableFst, Q: Queue>(
    fst: &mut F,
    opts: RmEpsilonConfig<F::W, Q>,
) -> Result<()>
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
        for tr in fst.tr_iter(state)? {
            if tr.ilabel != EPS_LABEL || tr.olabel != EPS_LABEL {
                noneps_in[tr.nextstate] = true;
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
        dfs_visit(fst, &mut visitor, &EpsilonTrFilter {}, false);

        for i in 0..visitor.order.len() {
            states[visitor.order[i]] = i;
        }
    } else {
        let mut visitor = SccVisitor::new(fst, true, false);
        dfs_visit(fst, &mut visitor, &EpsilonTrFilter {}, false);

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

    let mut rmeps_state = RmEpsilonState::<F, _, _>::new(&*fst, opts);
    let zero = F::W::zero();

    let mut v: Vec<(_, (_, F::W))> = Vec::with_capacity(states.len());
    for state in states.into_iter().rev() {
        if !noneps_in[state] {
            continue;
        }
        let (trs, final_weight) = rmeps_state.expand(state)?;

        // Copy everything, not great ...
        v.push((state, (trs, final_weight)));
    }

    for (state, (trs, final_weight)) in v.into_iter() {
        unsafe {
            // TODO: Use these trs instead of cloning
            fst.pop_trs_unchecked(state);
            fst.set_trs_unchecked(state, trs.into_iter().rev().collect());
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
                fst.delete_trs(s)?;
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

#[derive(Clone, Eq)]
struct RmEpsilonState<F: MutableFst, B: Borrow<F>, Q: Queue> {
    visited: Vec<bool>,
    visited_states: Vec<StateId>,
    element_map: HashMap<Element, (StateId, usize)>,
    expand_id: usize,
    sd_state: ShortestDistanceState<Q, F, B, EpsilonTrFilter>,
}

impl<F: MutableFst, B: Borrow<F>, Q: Queue> std::fmt::Debug for RmEpsilonState<F, B, Q> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RmEpsilonState {{ visited : {:?}, visited_states : {:?}, element_map : {:?}, expand_id : {:?}, sd_state : {:?} }}",
        self.visited, self.visited_states, self.element_map, self.expand_id, self.sd_state)
    }
}

impl<F: MutableFst, B: Borrow<F>, Q: Queue + PartialEq> PartialEq for RmEpsilonState<F, B, Q> {
    fn eq(&self, other: &Self) -> bool {
        self.visited.eq(&other.visited)
            && self.visited_states.eq(&other.visited_states)
            && self.element_map.eq(&other.element_map)
            && self.expand_id.eq(&other.expand_id)
            && self.sd_state.eq(&other.sd_state)
    }
}

impl<F: MutableFst, B: Borrow<F>, Q: Queue> RmEpsilonState<F, B, Q>
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

    pub fn expand(&mut self, source: StateId) -> Result<(Vec<Tr<F::W>>, F::W)> {
        let zero = F::W::zero();
        let distance = self.sd_state.shortest_distance(Some(source))?;

        let tr_filter = EpsilonTrFilter {};

        let mut eps_queue = vec![source];

        let mut trs = vec![];
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
            for tr in self.sd_state.fst.borrow().tr_iter(state)? {
                // TODO: Remove this clone
                let mut tr = tr.clone();
                tr.weight = distance[state].times(&tr.weight)?;
                if tr_filter.keep(&tr) {
                    while self.visited.len() <= tr.nextstate {
                        self.visited.push(false);
                    }
                    if !self.visited[tr.nextstate] {
                        eps_queue.push(tr.nextstate);
                    }
                } else {
                    let elt = Element {
                        ilabel: tr.ilabel,
                        olabel: tr.olabel,
                        nextstate: tr.nextstate,
                    };
                    let val = (self.expand_id, trs.len());

                    match self.element_map.entry(elt) {
                        Entry::Vacant(e) => {
                            e.insert(val);
                            trs.push(tr);
                        }
                        Entry::Occupied(mut e) => {
                            if e.get().0 == self.expand_id {
                                unsafe {
                                    trs.get_unchecked_mut(e.get().1)
                                        .weight
                                        .plus_assign(&tr.weight)?;
                                }
                            } else {
                                e.get_mut().0 = self.expand_id;
                                e.get_mut().1 = trs.len();
                                trs.push(tr);
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

        Ok((trs, final_weight))
    }
}

#[derive(Clone, Eq)]
pub struct RmEpsilonImpl<F: MutableFst, B: Borrow<F>> {
    rmeps_state: RmEpsilonState<F, B, FifoQueue>,
    cache_impl: CacheImpl<F::W>,
}

impl<F: MutableFst, B: Borrow<F>> std::fmt::Debug for RmEpsilonImpl<F, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RmEpsilonImpl {{ rmeps_state : {:?}, cache_impl : {:?} }}",
            self.rmeps_state, self.cache_impl
        )
    }
}

impl<F: MutableFst, B: Borrow<F>> PartialEq for RmEpsilonImpl<F, B> {
    fn eq(&self, other: &Self) -> bool {
        self.rmeps_state.eq(&other.rmeps_state) && self.cache_impl.eq(&other.cache_impl)
    }
}

impl<F: MutableFst, B: Borrow<F>> RmEpsilonImpl<F, B>
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

impl<F: MutableFst, B: Borrow<F>> FstImpl for RmEpsilonImpl<F, B>
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

    fn expand(&mut self, state: usize) -> Result<()> {
        let (trs, final_weight) = self.rmeps_state.expand(state)?;
        let zero = F::W::zero();

        for tr in trs.into_iter().rev() {
            self.cache_impl.push_tr(state, tr)?;
        }
        if final_weight != zero {
            self.cache_impl
                .set_final_weight(state, Some(final_weight))?;
        } else {
            self.cache_impl.set_final_weight(state, None)?;
        }

        Ok(())
    }

    fn compute_start(&mut self) -> Result<Option<usize>> {
        Ok(self.rmeps_state.sd_state.fst.borrow().start())
    }

    fn compute_final(&mut self, state: usize) -> Result<Option<Self::W>> {
        // A bit hacky as the final weight is computed inside the expand function.
        // Should in theory never be called
        self.expand(state)?;
        let weight = self.cache_impl.final_weight(state)?;
        Ok(weight.cloned())
    }
}

/// Removes epsilon-transitions (when both the input and output label are an
/// epsilon) from a transducer. The result will be an equivalent FST that has no
/// such epsilon transitions. This version is a delayed FST.
pub type RmEpsilonFst<F, B> = LazyFst<RmEpsilonImpl<F, B>>;
impl<F: MutableFst, B: Borrow<F>> RmEpsilonFst<F, B>
where
    <<F as CoreFst>::W as Semiring>::ReverseWeight: 'static,
    F::W: 'static,
{
    pub fn new(fst: B) -> Self {
        let isymt = fst.borrow().input_symbols();
        let osymt = fst.borrow().output_symbols();
        Self::from_impl(RmEpsilonImpl::new(fst), isymt, osymt)
    }
}
