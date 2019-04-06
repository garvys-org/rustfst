use std::collections::HashSet;

use bitflags::bitflags;

use failure::Fallible;

use crate::algorithms::{dfs, find_strongly_connected_components};
use crate::fst_traits::Fst;
use crate::Arc;
use crate::semirings::Semiring;

bitflags! {
    /// The property bits here assert facts about an FST. If individual bits are
    /// added, then the composite properties below, the property functions and
    /// property names in properties.cc, and TestProperties() in test-properties.h
    /// should be updated.
    /// For each of these properties below there is a pair of property bits, one
    /// positive and one negative. If the positive bit is set, the property is true.
    /// If the negative bit is set, the property is false. If neither is set, the
    /// property has unknown value. Both should never be simultaneously set. The
    /// individual positive and negative bit pairs should be adjacent with the
    /// positive bit at an odd and lower position.
    pub struct FstProperties: u32 {
        /// ilabel == olabel for each arc.
        const ACCEPTOR = 0b1;
        /// ilabel != olabel for some arc.
        const NOT_ACCEPTOR = 0b1 << 1;

        /// ilabels unique leaving each state.
        const I_DETERMINISTIC = 0b1 << 2;
        /// ilabels not unique leaving some state.
        const NOT_I_DETERMINISTIC = 0b1 << 3;

        /// olabels unique leaving each state.
        const O_DETERMINISTIC = 0b1 << 4;
        /// olabels not unique leaving some state.
        const NOT_O_DETERMINISTIC = 0b1 << 5;

        /// FST has input/output epsilons.
        const EPSILONS = 0b1 << 6;
        /// FST has no input/output epsilons.
        const NO_EPSILONS = 0b1 << 7;

        /// FST has input epsilons.
        const I_EPSILONS = 0b1 << 8;
        /// FST has no input epsilons.
        const NO_I_EPSILONS = 0b1 << 9;

        /// FST has output epsilons.
        const O_EPSILONS = 0b1 << 10;
        /// FST has no output epsilons.
        const NO_O_EPSILONS = 0b1 << 11;

        /// ilabels sorted wrt < for each state.
        const I_LABEL_SORTED = 0b1 << 12;
        /// ilabels not sorted wrt < for some state.
        const NOT_I_LABEL_SORTED = 0b1 << 13;

        /// olabels sorted wrt < for each state.
        const O_LABEL_SORTED = 0b1 << 14;
        /// olabels not sorted wrt < for some state.
        const NOT_O_LABEL_SORTED = 0b1 << 15;

        /// Non-trivial arc or final weights.
        const WEIGHTED = 0b1 << 16;
        /// Only trivial arc and final weights.
        const UNWEIGHTED = 0b1 << 17;

        /// FST has cycles.
        const CYCLIC = 0b1 << 18;
        /// FST has no cycles.
        const ACYCLIC = 0b1 << 29;

        /// FST has cycles containing the initial state.
        const INITIAL_CYCLIC = 0b1 << 20;
        /// FST has no cycles containing the initial state.
        const INITIAL_ACYCLIC = 0b1 << 21;

        /// FST is topologically sorted.
        const TOP_SORTED = 0b1 << 22;
        /// FST is not topologically sorted.
        const NOT_TOP_SORTED = 0b1 << 23;

        /// All states reachable from the initial state.
        const ACCESSIBLE = 0b1 << 24;
        /// Not all states reachable from the initial state.
        const NOT_ACCESSIBLE = 0b1 << 25;

        /// All states can reach a final state.
        const COACCESSIBLE = 0b1 << 26;
        /// Not all states can reach a final state.
        const NOT_COACCESSIBLE = 0b1 << 27;

        /// If NumStates() > 0, then state 0 is initial, state NumStates() - 1 is final,
        /// there is a transition from each non-final state i to state i + 1, and there
        /// are no other transitions.
        const STRING = 0b1 << 28;

        /// Not a string FST.
        const NOT_STRING = 0b1 << 29;

        /// FST has at least one weighted cycle.
        const WEIGHTED_CYCLES = 0b1 << 30;

        /// Only unweighted cycles.
        const UNWEIGHTED_CYCLES = 0b1 << 31;

        /// Properties of an empty machine.
        const NULL_PROPERTIES =
            Self::ACCEPTOR.bits | Self::I_DETERMINISTIC.bits | Self::O_DETERMINISTIC.bits |
            Self::NO_EPSILONS.bits | Self::NO_I_EPSILONS.bits | Self::NO_O_EPSILONS.bits |
            Self::I_LABEL_SORTED.bits | Self::O_LABEL_SORTED.bits | Self::UNWEIGHTED.bits |
            Self::ACYCLIC.bits | Self::INITIAL_ACYCLIC.bits | Self::TOP_SORTED.bits |
            Self::ACCESSIBLE.bits | Self::COACCESSIBLE.bits | Self::STRING.bits |
            Self::UNWEIGHTED_CYCLES.bits;

        /// Properties that are preserved when an FST is copied.
        const COPY_PROPERTIES =
            Self::ACCEPTOR.bits | Self::NOT_ACCEPTOR.bits | Self::I_DETERMINISTIC.bits |
            Self::NOT_I_DETERMINISTIC.bits | Self::O_DETERMINISTIC.bits |
            Self::NOT_O_DETERMINISTIC.bits | Self::EPSILONS.bits | Self::NO_EPSILONS.bits |
            Self::I_EPSILONS.bits | Self::NO_I_EPSILONS.bits | Self::O_EPSILONS.bits |
            Self::NO_O_EPSILONS.bits | Self::I_LABEL_SORTED.bits | Self::NOT_I_LABEL_SORTED.bits |
            Self::O_LABEL_SORTED.bits | Self::NOT_O_LABEL_SORTED.bits | Self::WEIGHTED.bits |
            Self::UNWEIGHTED.bits | Self::CYCLIC.bits | Self::ACYCLIC.bits |
            Self::INITIAL_CYCLIC.bits | Self::INITIAL_ACYCLIC.bits | Self::TOP_SORTED.bits |
            Self::NOT_TOP_SORTED.bits | Self::ACCESSIBLE.bits | Self::NOT_ACCESSIBLE.bits |
            Self::COACCESSIBLE.bits | Self::NOT_COACCESSIBLE.bits | Self::STRING.bits |
            Self::NOT_STRING.bits | Self::WEIGHTED_CYCLES.bits | Self::UNWEIGHTED_CYCLES.bits;

        /// Properties that are intrinsic to the FST.
        const INTRINSIC_PROPERTIES =
            Self::ACCEPTOR.bits | Self::NOT_ACCEPTOR.bits | Self::I_DETERMINISTIC.bits |
            Self::NOT_I_DETERMINISTIC.bits | Self::O_DETERMINISTIC.bits |
            Self::NOT_O_DETERMINISTIC.bits | Self::EPSILONS.bits | Self::NO_EPSILONS.bits |
            Self::I_EPSILONS.bits | Self::NO_I_EPSILONS.bits | Self::O_EPSILONS.bits |
            Self::NO_O_EPSILONS.bits | Self::I_LABEL_SORTED.bits | Self::NOT_I_LABEL_SORTED.bits |
            Self::O_LABEL_SORTED.bits | Self::NOT_O_LABEL_SORTED.bits | Self::WEIGHTED.bits |
            Self::UNWEIGHTED.bits | Self::CYCLIC.bits | Self::ACYCLIC.bits |
            Self::INITIAL_CYCLIC.bits | Self::INITIAL_ACYCLIC.bits | Self::TOP_SORTED.bits |
            Self::NOT_TOP_SORTED.bits | Self::ACCESSIBLE.bits | Self::NOT_ACCESSIBLE.bits |
            Self::COACCESSIBLE.bits | Self::NOT_COACCESSIBLE.bits | Self::STRING.bits |
            Self::NOT_STRING.bits | Self::WEIGHTED_CYCLES.bits | Self::UNWEIGHTED_CYCLES.bits;

        /// Properties that are preserved when an FST start state is set.
        const SET_START_PROPERTIES =
            Self::ACCEPTOR.bits | Self::NOT_ACCEPTOR.bits | Self::I_DETERMINISTIC.bits |
            Self::NOT_I_DETERMINISTIC.bits | Self::O_DETERMINISTIC.bits |
            Self::NOT_O_DETERMINISTIC.bits | Self::EPSILONS.bits | Self::NO_EPSILONS.bits |
            Self::I_EPSILONS.bits | Self::NO_I_EPSILONS.bits | Self::O_EPSILONS.bits |
            Self::NO_O_EPSILONS.bits | Self::I_LABEL_SORTED.bits | Self::NOT_I_LABEL_SORTED.bits |
            Self::O_LABEL_SORTED.bits | Self::NOT_O_LABEL_SORTED.bits | Self::WEIGHTED.bits |
            Self::UNWEIGHTED.bits | Self::CYCLIC.bits | Self::ACYCLIC.bits | Self::TOP_SORTED.bits |
            Self::NOT_TOP_SORTED.bits | Self::COACCESSIBLE.bits | Self::NOT_COACCESSIBLE.bits |
            Self::WEIGHTED_CYCLES.bits | Self::UNWEIGHTED_CYCLES.bits;

        /// Properties that are preserved when an FST final weight is set.
        const SET_FINAL_PROPERTIES =
            Self::ACCEPTOR.bits | Self::NOT_ACCEPTOR.bits | Self::I_DETERMINISTIC.bits |
            Self::NOT_I_DETERMINISTIC.bits | Self::O_DETERMINISTIC.bits |
            Self::NOT_O_DETERMINISTIC.bits | Self::EPSILONS.bits | Self::NO_EPSILONS.bits |
            Self::I_EPSILONS.bits | Self::NO_I_EPSILONS.bits | Self::O_EPSILONS.bits |
            Self::NO_O_EPSILONS.bits | Self::I_LABEL_SORTED.bits | Self::NOT_I_LABEL_SORTED.bits |
            Self::O_LABEL_SORTED.bits | Self::NOT_O_LABEL_SORTED.bits | Self::CYCLIC.bits |
            Self::ACYCLIC.bits | Self::INITIAL_CYCLIC.bits | Self::INITIAL_ACYCLIC.bits |
            Self::TOP_SORTED.bits | Self::NOT_TOP_SORTED.bits | Self::ACCESSIBLE.bits |
            Self::NOT_ACCESSIBLE.bits | Self::WEIGHTED_CYCLES.bits | Self::UNWEIGHTED_CYCLES.bits;

        /// Properties that are preserved when an FST state is added.
        const ADD_STATE_PROPERTIES =
            Self::ACCEPTOR.bits | Self::NOT_ACCEPTOR.bits | Self::I_DETERMINISTIC.bits |
            Self::NOT_I_DETERMINISTIC.bits | Self::O_DETERMINISTIC.bits |
            Self::NOT_O_DETERMINISTIC.bits | Self::EPSILONS.bits | Self::NO_EPSILONS.bits |
            Self::I_EPSILONS.bits | Self::NO_I_EPSILONS.bits | Self::O_EPSILONS.bits |
            Self::NO_O_EPSILONS.bits | Self::I_LABEL_SORTED.bits | Self::NOT_I_LABEL_SORTED.bits |
            Self::O_LABEL_SORTED.bits | Self::NOT_O_LABEL_SORTED.bits | Self::WEIGHTED.bits |
            Self::UNWEIGHTED.bits | Self::CYCLIC.bits | Self::ACYCLIC.bits |
            Self::INITIAL_CYCLIC.bits | Self::INITIAL_ACYCLIC.bits | Self::TOP_SORTED.bits |
            Self::NOT_TOP_SORTED.bits | Self::NOT_ACCESSIBLE.bits | Self::NOT_COACCESSIBLE.bits |
            Self::NOT_STRING.bits | Self::WEIGHTED_CYCLES.bits | Self::UNWEIGHTED_CYCLES.bits;

        /// Properties that are preserved when an FST arc is added.
        const ADD_ARC_PROPERTIES =
            Self::NOT_ACCEPTOR.bits | Self::NOT_I_DETERMINISTIC.bits |
            Self::NOT_O_DETERMINISTIC.bits | Self::EPSILONS.bits | Self::I_EPSILONS.bits |
            Self::O_EPSILONS.bits | Self::NOT_I_LABEL_SORTED.bits | Self::NOT_O_LABEL_SORTED.bits |
            Self::WEIGHTED.bits | Self::CYCLIC.bits | Self::INITIAL_CYCLIC.bits |
            Self::NOT_TOP_SORTED.bits | Self::ACCESSIBLE.bits | Self::COACCESSIBLE.bits |
            Self::WEIGHTED_CYCLES.bits;

        /// Properties that are preserved when an FST arc is set.
        const SET_ARC_PROPERTIES = 0b0;

        /// Properties that are preserved when FST states are deleted.
        const DELETE_STATE_PROPERTIES =
            Self::ACCEPTOR.bits | Self::I_DETERMINISTIC.bits |
            Self::O_DETERMINISTIC.bits | Self::NO_EPSILONS.bits | Self::NO_I_EPSILONS.bits |
            Self::NO_O_EPSILONS.bits | Self::I_LABEL_SORTED.bits | Self::O_LABEL_SORTED.bits |
            Self::UNWEIGHTED.bits | Self::ACYCLIC.bits | Self::INITIAL_ACYCLIC.bits |
            Self::TOP_SORTED.bits | Self::UNWEIGHTED_CYCLES.bits;

        /// Properties that are preserved when FST arcs are deleted.
        const DELETE_ARCS_PROPERTIES =
            Self::ACCEPTOR.bits | Self::I_DETERMINISTIC.bits |
            Self::O_DETERMINISTIC.bits | Self::NO_EPSILONS.bits | Self::NO_I_EPSILONS.bits |
            Self::NO_O_EPSILONS.bits | Self::I_LABEL_SORTED.bits | Self::O_LABEL_SORTED.bits |
            Self::UNWEIGHTED.bits | Self::ACYCLIC.bits | Self::INITIAL_ACYCLIC.bits |
            Self::TOP_SORTED.bits | Self::NOT_ACCESSIBLE.bits | Self::NOT_COACCESSIBLE.bits |
            Self::UNWEIGHTED_CYCLES.bits;

        /// Properties that are preserved when an FST's states are reordered.
        const STATESORT_PROPERTIES =
            Self::ACCEPTOR.bits | Self::NOT_ACCEPTOR.bits | Self::I_DETERMINISTIC.bits |
            Self::NOT_I_DETERMINISTIC.bits | Self::O_DETERMINISTIC.bits |
            Self::NOT_O_DETERMINISTIC.bits | Self::EPSILONS.bits | Self::NO_EPSILONS.bits |
            Self::I_EPSILONS.bits | Self::NO_I_EPSILONS.bits | Self::O_EPSILONS.bits |
            Self::NO_O_EPSILONS.bits | Self::I_LABEL_SORTED.bits | Self::NOT_I_LABEL_SORTED.bits |
            Self::O_LABEL_SORTED.bits | Self::NOT_O_LABEL_SORTED.bits | Self::WEIGHTED.bits |
            Self::UNWEIGHTED.bits | Self::CYCLIC.bits | Self::ACYCLIC.bits |
            Self::INITIAL_CYCLIC.bits | Self::INITIAL_ACYCLIC.bits | Self::ACCESSIBLE.bits |
            Self::NOT_ACCESSIBLE.bits | Self::COACCESSIBLE.bits | Self::NOT_COACCESSIBLE.bits |
            Self::WEIGHTED_CYCLES.bits | Self::UNWEIGHTED_CYCLES.bits;

        /// Properties that are preserved when an FST's arcs are reordered.
        const ARCSORT_PROPERTIES =
            Self::ACCEPTOR.bits | Self::NOT_ACCEPTOR.bits | Self::I_DETERMINISTIC.bits |
            Self::NOT_I_DETERMINISTIC.bits | Self::O_DETERMINISTIC.bits |
            Self::NOT_O_DETERMINISTIC.bits | Self::EPSILONS.bits | Self::NO_EPSILONS.bits |
            Self::I_EPSILONS.bits | Self::NO_I_EPSILONS.bits | Self::O_EPSILONS.bits |
            Self::NO_O_EPSILONS.bits | Self::WEIGHTED.bits | Self::UNWEIGHTED.bits |
            Self::CYCLIC.bits | Self::ACYCLIC.bits | Self::INITIAL_CYCLIC.bits |
            Self::INITIAL_ACYCLIC.bits | Self::TOP_SORTED.bits | Self::NOT_TOP_SORTED.bits |
            Self::ACCESSIBLE.bits | Self::NOT_ACCESSIBLE.bits | Self::COACCESSIBLE.bits |
            Self::NOT_COACCESSIBLE.bits | Self::STRING.bits | Self::NOT_STRING.bits |
            Self::WEIGHTED_CYCLES.bits | Self::UNWEIGHTED_CYCLES.bits;

        /// Properties that are preserved when an FST's input labels are changed.
        const I_LABEL_INVARIANT_PROPERTIES =
            Self::O_DETERMINISTIC.bits | Self::NOT_O_DETERMINISTIC.bits |
            Self::O_EPSILONS.bits | Self::NO_O_EPSILONS.bits | Self::O_LABEL_SORTED.bits |
            Self::NOT_O_LABEL_SORTED.bits | Self::WEIGHTED.bits | Self::UNWEIGHTED.bits |
            Self::CYCLIC.bits | Self::ACYCLIC.bits | Self::INITIAL_CYCLIC.bits |
            Self::INITIAL_ACYCLIC.bits | Self::TOP_SORTED.bits | Self::NOT_TOP_SORTED.bits |
            Self::ACCESSIBLE.bits | Self::NOT_ACCESSIBLE.bits | Self::COACCESSIBLE.bits |
            Self::NOT_COACCESSIBLE.bits | Self::STRING.bits | Self::NOT_STRING.bits |
            Self::WEIGHTED_CYCLES.bits | Self::UNWEIGHTED_CYCLES.bits;

        /// Properties that are preserved when an FST's output labels are changed.
        const O_LABEL_INVARIANT_PROPERTIES =
            Self::I_DETERMINISTIC.bits | Self::NOT_I_DETERMINISTIC.bits | Self::I_EPSILONS.bits |
            Self::NO_I_EPSILONS.bits | Self::I_LABEL_SORTED.bits | Self::NOT_I_LABEL_SORTED.bits |
            Self::WEIGHTED.bits | Self::UNWEIGHTED.bits | Self::CYCLIC.bits | Self::ACYCLIC.bits |
            Self::INITIAL_CYCLIC.bits | Self::INITIAL_ACYCLIC.bits | Self::TOP_SORTED.bits |
            Self::NOT_TOP_SORTED.bits | Self::ACCESSIBLE.bits | Self::NOT_ACCESSIBLE.bits |
            Self::COACCESSIBLE.bits | Self::NOT_COACCESSIBLE.bits | Self::STRING.bits |
            Self::NOT_STRING.bits | Self::WEIGHTED_CYCLES.bits | Self::UNWEIGHTED_CYCLES.bits;

        /// Properties that are preserved when an FST's weights are changed. This
        /// assumes that the set of states that are non-final is not changed.
        const WEIGHT_INVARIANT_PROPERTIES =
            Self::ACCEPTOR.bits | Self::NOT_ACCEPTOR.bits | Self::I_DETERMINISTIC.bits |
            Self::NOT_I_DETERMINISTIC.bits | Self::O_DETERMINISTIC.bits |
            Self::NOT_O_DETERMINISTIC.bits | Self::EPSILONS.bits | Self::NO_EPSILONS.bits |
            Self::I_EPSILONS.bits | Self::NO_I_EPSILONS.bits | Self::O_EPSILONS.bits |
            Self::NO_O_EPSILONS.bits | Self::I_LABEL_SORTED.bits | Self::NOT_I_LABEL_SORTED.bits |
            Self::O_LABEL_SORTED.bits | Self::NOT_O_LABEL_SORTED.bits | Self::CYCLIC.bits |
            Self::ACYCLIC.bits | Self::INITIAL_CYCLIC.bits | Self::INITIAL_ACYCLIC.bits |
            Self::TOP_SORTED.bits | Self::NOT_TOP_SORTED.bits | Self::ACCESSIBLE.bits |
            Self::NOT_ACCESSIBLE.bits | Self::COACCESSIBLE.bits | Self::NOT_COACCESSIBLE.bits |
            Self::STRING.bits | Self::NOT_STRING.bits;

        /// Properties that are preserved when a superfinal state is added and an FST's
        /// final weights are directed to it via new transitions.
        const ADD_SUPER_FINAL_PROPERTIES =
            Self::NOT_ACCEPTOR.bits |
            Self::NOT_I_DETERMINISTIC.bits | Self::NOT_O_DETERMINISTIC.bits | Self::EPSILONS.bits |
            Self::I_EPSILONS.bits | Self::O_EPSILONS.bits | Self::NOT_I_LABEL_SORTED.bits |
            Self::NOT_O_LABEL_SORTED.bits | Self::WEIGHTED.bits | Self::UNWEIGHTED.bits |
            Self::CYCLIC.bits | Self::ACYCLIC.bits | Self::INITIAL_CYCLIC.bits |
            Self::INITIAL_ACYCLIC.bits | Self::NOT_TOP_SORTED.bits | Self::NOT_ACCESSIBLE.bits |
            Self::COACCESSIBLE.bits | Self::NOT_COACCESSIBLE.bits | Self::NOT_STRING.bits |
            Self::WEIGHTED_CYCLES.bits | Self::UNWEIGHTED_CYCLES.bits;

        /// Properties that are preserved when a superfinal state is removed and the
        /// epsilon transitions directed to it are made final weights.
        const RM_SUPER_FINAL_PROPERTIES =
            Self::ACCEPTOR.bits | Self::NOT_ACCEPTOR.bits | Self::I_DETERMINISTIC.bits |
            Self::O_DETERMINISTIC.bits | Self::NO_EPSILONS.bits | Self::NO_I_EPSILONS.bits |
            Self::NO_O_EPSILONS.bits | Self::I_LABEL_SORTED.bits | Self::O_LABEL_SORTED.bits |
            Self::WEIGHTED.bits | Self::UNWEIGHTED.bits | Self::CYCLIC.bits | Self::ACYCLIC.bits |
            Self::INITIAL_CYCLIC.bits | Self::INITIAL_ACYCLIC.bits | Self::TOP_SORTED.bits |
            Self::ACCESSIBLE.bits | Self::COACCESSIBLE.bits | Self::NOT_COACCESSIBLE.bits |
            Self::STRING.bits | Self::WEIGHTED_CYCLES.bits | Self::UNWEIGHTED_CYCLES.bits;

        const POS_PROPERTIES = 0b0101_0101_0101_0101_0101_0101_0101_0101;
        const NEG_PROPERTIES = Self::POS_PROPERTIES.bits << 1;
        const ALL_PROPERTIES = Self::POS_PROPERTIES.bits | Self::NEG_PROPERTIES.bits;
    }

}

pub fn compute_fst_properties<F: Fst>(fst: &F) -> Fallible<FstProperties> {
    let states: Vec<_> = fst.states_iter().collect();
    let mut comp_props = FstProperties::ALL_PROPERTIES;
    let mut accessible_states = HashSet::new();
    let mut coaccessible_states = HashSet::new();

    if let Some(start) = fst.start() {
        dfs(fst, start, &mut accessible_states, &mut coaccessible_states)?;
    }

    comp_props |= FstProperties::ACCESSIBLE;
    if accessible_states.len() != states.len() {
        // All states are not accessible
        comp_props |= FstProperties::NOT_ACCESSIBLE;
        comp_props &= !FstProperties::ACCESSIBLE;
    }

    comp_props |= FstProperties::COACCESSIBLE;
    if coaccessible_states.len() != states.len() {
        // All states are not coaccessible
        comp_props |= FstProperties::NOT_COACCESSIBLE;
        comp_props &= !FstProperties::COACCESSIBLE;
    }

    let mut sccs = vec![];
    let mut n_sccs = 0;
    find_strongly_connected_components(fst, &mut sccs, &mut n_sccs)?;

    comp_props |= FstProperties::ACYCLIC;
    comp_props |= FstProperties::INITIAL_ACYCLIC;
    if n_sccs < states.len() {
        // Cycles
        comp_props |= FstProperties::CYCLIC;
        comp_props &= !FstProperties::ACYCLIC;

        if let Some(start) = fst.start() {
            if sccs.iter().any(|s| sccs[*s] == sccs[start]) {
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

            if ! arc.weight.is_one() && !arc.weight.is_zero() {
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
        if fst.is_final(state) {
            let final_weight = fst.final_weight(state).unwrap();
            if !final_weight.is_one() {
                comp_props |= FstProperties::WEIGHTED;
                comp_props &= !FstProperties::UNWEIGHTED;
            }
            nfinal += 1;
        } else {
            if fst.num_arcs(state)? != 1 {
                comp_props |= FstProperties::NOT_STRING;
                comp_props &= !FstProperties::STRING;
            }
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
