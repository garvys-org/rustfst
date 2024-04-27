use anyhow::Result;

use crate::algorithms::dfs_visit::dfs_visit;
use crate::algorithms::queues::AutoQueue;
use crate::algorithms::rm_epsilon::{RmEpsilonInternalConfig, RmEpsilonState};
use crate::algorithms::top_sort::TopOrderVisitor;
use crate::algorithms::tr_filters::EpsilonTrFilter;
use crate::algorithms::visitors::SccVisitor;
use crate::algorithms::Queue;
use crate::fst_properties::mutable_properties::rmepsilon_properties;
use crate::fst_properties::FstProperties;
use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::{StateId, Trs, EPS_LABEL};

/// This operation removes epsilon-transitions (when both the input and
/// output labels are an epsilon) from a transducer. The result will be an
/// equivalent FST that has no such epsilon transitions.
///
/// # Example 1
/// ```
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::fst_traits::MutableFst;
/// # use rustfst::algorithms::rm_epsilon::rm_epsilon;
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
pub fn rm_epsilon<W: Semiring, F: MutableFst<W>>(fst: &mut F) -> Result<()> {
    let tr_filter = EpsilonTrFilter {};
    let queue = AutoQueue::new(fst, None, &tr_filter)?;
    let opts = RmEpsilonInternalConfig::new_with_default(queue);
    rm_epsilon_with_internal_config(fst, opts)
}
pub(crate) fn rm_epsilon_with_internal_config<W: Semiring, F: MutableFst<W>, Q: Queue>(
    fst: &mut F,
    opts: RmEpsilonInternalConfig<W, Q>,
) -> Result<()> {
    let connect = opts.connect;
    let weight_threshold = opts.weight_threshold.clone();
    let state_threshold = opts.state_threshold;

    let start_state = match fst.start() {
        None => return Ok(()),
        Some(s) => s,
    };

    // noneps_in[s] will be set to true iff s admits a non-epsilon incoming
    // transition or is the start state.
    let mut noneps_in = vec![false; fst.num_states()];
    noneps_in[start_state as usize] = true;

    for state in fst.states_iter() {
        for tr in fst.get_trs(state)?.trs() {
            if tr.ilabel != EPS_LABEL || tr.olabel != EPS_LABEL {
                noneps_in[tr.nextstate as usize] = true;
            }
        }
    }

    // States sorted in topological order when (acyclic) or generic topological
    // order (cyclic).
    let mut states = vec![];

    let fst_props = fst.properties();

    if fst_props.contains(FstProperties::TOP_SORTED) {
        states = fst.states_iter().collect();
    } else if fst_props.contains(FstProperties::ACYCLIC) {
        let mut visitor = TopOrderVisitor::new();
        dfs_visit(fst, &mut visitor, &EpsilonTrFilter {}, false);

        states.resize(visitor.order.len(), 0);
        for i in 0..visitor.order.len() {
            states[visitor.order[i] as usize] = i as StateId;
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

        for mut opt_j in &first {
            while let Some(j) = opt_j {
                states.push(*j as StateId);
                opt_j = &next[*j];
            }
        }
    }

    let mut rmeps_state = RmEpsilonState::new(fst.num_states(), opts);
    let zero = W::zero();

    for state in states.into_iter().rev() {
        if !noneps_in[state as usize]
            && (connect || weight_threshold != W::zero() || state_threshold.is_some())
        {
            continue;
        }
        let (trs, final_weight) = rmeps_state.expand::<F, _>(state, &*fst)?;

        unsafe {
            fst.pop_trs_unchecked(state);
            fst.set_trs_unchecked(state, trs.into_iter().rev().collect());
            if final_weight != zero {
                fst.set_final_unchecked(state, final_weight);
            } else {
                fst.delete_final_weight_unchecked(state);
            }
        }
    }

    if connect || weight_threshold != W::zero() || state_threshold.is_some() {
        for s in 0..(fst.num_states() as StateId) {
            if !noneps_in[s as usize] {
                fst.delete_trs(s)?;
            }
        }
    }

    fst.set_properties(rmepsilon_properties(fst.properties(), false));

    if weight_threshold != W::zero() || state_threshold.is_some() {
        todo!("Implement Prune!")
    }

    if connect && weight_threshold == W::zero() && state_threshold.is_none() {
        crate::algorithms::connect(fst)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fst_traits::Fst;
    use crate::prelude::{TropicalWeight, VectorFst};
    use crate::SymbolTable;
    use proptest::prelude::any;
    use proptest::proptest;
    use std::sync::Arc;

    proptest! {
        #[test]
        fn test_proptest_rmepsilon_keeps_symts(mut fst in any::<VectorFst::<TropicalWeight>>()) {
            let symt = Arc::new(SymbolTable::new());
            fst.set_input_symbols(Arc::clone(&symt));
            fst.set_output_symbols(Arc::clone(&symt));

            rm_epsilon(&mut fst).unwrap();

            assert!(fst.input_symbols().is_some());
            assert!(fst.output_symbols().is_some());
        }
    }
}
