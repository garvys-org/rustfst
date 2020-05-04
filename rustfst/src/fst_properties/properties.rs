use std::ops::{Shl, Shr};

use bitflags::bitflags;

bitflags! {
    /// The property bits here assert facts about an FST. If individual bits are
    /// added, then the composite fst_properties below, the property functions and
    /// property names in fst_properties.cc, and TestProperties() in test-fst_properties.h
    /// should be updated.
    /// For each of these fst_properties below there is a pair of property bits, one
    /// positive and one negative. If the positive bit is set, the property is true.
    /// If the negative bit is set, the property is false. If neither is set, the
    /// property has unknown value. Both should never be simultaneously set. The
    /// individual positive and negative bit pairs should be adjacent with the
    /// positive bit at an odd and lower position.
    pub struct FstProperties: u32 {
        /// ilabel == olabel for each transition.
        const ACCEPTOR = 0b1;
        /// ilabel != olabel for some transition.
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

        /// Non-trivial transition or final weights.
        const WEIGHTED = 0b1 << 16;
        /// Only trivial transition and final weights.
        const UNWEIGHTED = 0b1 << 17;

        /// FST has cycles.
        const CYCLIC = 0b1 << 18;
        /// FST has no cycles.
        const ACYCLIC = 0b1 << 19;

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

        /// Properties that are preserved when an FST transition is added.
        const ADD_ARC_PROPERTIES =
            Self::NOT_ACCEPTOR.bits | Self::NOT_I_DETERMINISTIC.bits |
            Self::NOT_O_DETERMINISTIC.bits | Self::EPSILONS.bits | Self::I_EPSILONS.bits |
            Self::O_EPSILONS.bits | Self::NOT_I_LABEL_SORTED.bits | Self::NOT_O_LABEL_SORTED.bits |
            Self::WEIGHTED.bits | Self::CYCLIC.bits | Self::INITIAL_CYCLIC.bits |
            Self::NOT_TOP_SORTED.bits | Self::ACCESSIBLE.bits | Self::COACCESSIBLE.bits |
            Self::WEIGHTED_CYCLES.bits;

        /// Properties that are preserved when an FST transition is set.
        const SET_ARC_PROPERTIES = 0b0;

        /// Properties that are preserved when FST states are deleted.
        const DELETE_STATES_PROPERTIES =
            Self::ACCEPTOR.bits | Self::I_DETERMINISTIC.bits |
            Self::O_DETERMINISTIC.bits | Self::NO_EPSILONS.bits | Self::NO_I_EPSILONS.bits |
            Self::NO_O_EPSILONS.bits | Self::I_LABEL_SORTED.bits | Self::O_LABEL_SORTED.bits |
            Self::UNWEIGHTED.bits | Self::ACYCLIC.bits | Self::INITIAL_ACYCLIC.bits |
            Self::TOP_SORTED.bits | Self::UNWEIGHTED_CYCLES.bits;

        /// Properties that are preserved when FST trs are deleted.
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

        /// Properties that are preserved when an FST's trs are reordered.
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

impl Shl<usize> for FstProperties {
    type Output = FstProperties;

    fn shl(self, rhs: usize) -> Self::Output {
        Self::from_bits_truncate(self.bits() << rhs)
    }
}

impl Shr<usize> for FstProperties {
    type Output = FstProperties;

    fn shr(self, rhs: usize) -> Self::Output {
        Self::from_bits_truncate(self.bits() >> rhs)
    }
}
