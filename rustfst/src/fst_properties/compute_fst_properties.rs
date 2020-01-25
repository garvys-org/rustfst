use std::collections::HashSet;

use failure::Fallible;
use unsafe_unwrap::UnsafeUnwrap;

use crate::algorithms::dfs_visit::dfs_visit;
use crate::algorithms::visitors::SccVisitor;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{ExpandedFst, Fst};
use crate::semirings::Semiring;
use crate::Arc;

/// Computes all the FstProperties of the FST bit don't attach them to the FST.
pub fn compute_fst_properties<F: Fst + ExpandedFst>(fst: &F) -> Fallible<FstProperties> {
    let states: Vec<_> = fst.states_iter().collect();
    let mut comp_props = FstProperties::empty();

    let mut visitor = SccVisitor::new(fst, true, true);
    dfs_visit(fst, &mut visitor, false);
    let sccs = unsafe { &visitor.scc.unsafe_unwrap() };

    comp_props |= FstProperties::ACCESSIBLE;
    if unsafe { visitor.access.unsafe_unwrap().iter().any(|v| !*v) } {
        // All states are not accessible
        comp_props |= FstProperties::NOT_ACCESSIBLE;
        comp_props &= !FstProperties::ACCESSIBLE;
    }

    comp_props |= FstProperties::COACCESSIBLE;
    if visitor.coaccess.iter().any(|v| !*v) {
        // All states are not coaccessible
        comp_props |= FstProperties::NOT_COACCESSIBLE;
        comp_props &= !FstProperties::COACCESSIBLE;
    }

    comp_props |= FstProperties::ACYCLIC;
    comp_props |= FstProperties::INITIAL_ACYCLIC;
    if (visitor.nscc as usize) < states.len() {
        // Cycles
        comp_props |= FstProperties::CYCLIC;
        comp_props &= !FstProperties::ACYCLIC;

        if let Some(start) = fst.start() {
            if sccs.iter().any(|s| sccs[*s as usize] == sccs[start]) {
                // if the start state is not alone in its scc, then it is initial cyclic.
                comp_props |= FstProperties::INITIAL_CYCLIC;
                comp_props &= !FstProperties::INITIAL_ACYCLIC;
            }
        }
    }

    comp_props |= FstProperties::ACCEPTOR
        | FstProperties::NO_EPSILONS
        | FstProperties::NO_I_EPSILONS
        | FstProperties::NO_O_EPSILONS
        | FstProperties::I_LABEL_SORTED
        | FstProperties::O_LABEL_SORTED
        | FstProperties::UNWEIGHTED
        | FstProperties::TOP_SORTED
        | FstProperties::STRING
        | FstProperties::I_DETERMINISTIC
        | FstProperties::O_DETERMINISTIC
        | FstProperties::UNWEIGHTED_CYCLES;

    let mut nfinal = 0;
    for state in states {
        let mut ilabels = HashSet::new();
        let mut olabels = HashSet::new();
        let mut prev_arc: Option<&Arc<F::W>> = None;
        for arc in fst.arcs_iter(state)? {
            // There is already an outgoing arc with this ilabel
            if ilabels.contains(&arc.ilabel) {
                comp_props |= FstProperties::NOT_I_DETERMINISTIC;
                comp_props &= !FstProperties::I_DETERMINISTIC;
            }

            // There is already an outgoing arc with this olabel
            if olabels.contains(&arc.olabel) {
                comp_props |= FstProperties::NOT_O_DETERMINISTIC;
                comp_props &= !FstProperties::O_DETERMINISTIC;
            }

            if arc.ilabel != arc.olabel {
                comp_props |= FstProperties::NOT_ACCEPTOR;
                comp_props &= !FstProperties::ACCEPTOR;
            }

            if arc.ilabel == 0 && arc.olabel == 0 {
                comp_props |= FstProperties::EPSILONS;
                comp_props &= !FstProperties::NO_EPSILONS;
            }

            if arc.ilabel == 0 {
                comp_props |= FstProperties::I_EPSILONS;
                comp_props &= !FstProperties::NO_I_EPSILONS;
            }

            if arc.olabel == 0 {
                comp_props |= FstProperties::O_EPSILONS;
                comp_props &= !FstProperties::NO_O_EPSILONS;
            }

            // Not first arc
            if let Some(_prev_arc) = prev_arc {
                if arc.ilabel < _prev_arc.ilabel {
                    comp_props |= FstProperties::NOT_I_LABEL_SORTED;
                    comp_props &= !FstProperties::I_LABEL_SORTED;
                }

                if arc.olabel < _prev_arc.olabel {
                    comp_props |= FstProperties::NOT_O_LABEL_SORTED;
                    comp_props &= !FstProperties::O_LABEL_SORTED;
                }
            }

            if !arc.weight.is_one() && !arc.weight.is_zero() {
                comp_props |= FstProperties::WEIGHTED;
                comp_props &= !FstProperties::UNWEIGHTED;

                if sccs[state] == sccs[arc.nextstate] {
                    comp_props |= FstProperties::WEIGHTED_CYCLES;
                    comp_props &= !FstProperties::UNWEIGHTED_CYCLES;
                }
            }

            if arc.nextstate <= state {
                comp_props |= FstProperties::NOT_TOP_SORTED;
                comp_props &= !FstProperties::TOP_SORTED;
            }

            if state == arc.nextstate {
                comp_props |= FstProperties::CYCLIC;
                comp_props &= !FstProperties::ACYCLIC;
            }

            if arc.nextstate != state + 1 {
                comp_props |= FstProperties::NOT_STRING;
                comp_props &= !FstProperties::STRING;
            }

            prev_arc = Some(arc);
            ilabels.insert(arc.ilabel);
            olabels.insert(arc.olabel);
        }

        if nfinal > 0 {
            comp_props |= FstProperties::NOT_STRING;
            comp_props &= !FstProperties::STRING;
        }
        if fst.is_final(state)? {
            let final_weight = unsafe { fst.final_weight_unchecked(state).unsafe_unwrap() };
            if !final_weight.is_one() {
                comp_props |= FstProperties::WEIGHTED;
                comp_props &= !FstProperties::UNWEIGHTED;
            }
            nfinal += 1;
        } else if fst.num_arcs(state)? != 1 {
            comp_props |= FstProperties::NOT_STRING;
            comp_props &= !FstProperties::STRING;
        }
    }

    if let Some(start) = fst.start() {
        if start != 0 {
            comp_props |= FstProperties::NOT_STRING;
            comp_props &= !FstProperties::STRING;
        }
    }
    Ok(comp_props)
}
