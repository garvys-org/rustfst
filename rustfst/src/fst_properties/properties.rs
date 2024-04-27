use std::ops::{Shl, Shr};

use bitflags::bitflags;

pub(crate) const EXPANDED: u64 = 0x0000_0000_0000_0001;
pub(crate) const MUTABLE: u64 = 0x0000_0000_0000_0002;

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
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct FstProperties: u64 {
        /// ilabel == olabel for each transition.
        const ACCEPTOR = 0x0000_0000_0001_0000;
        /// ilabel != olabel for some transition.
        const NOT_ACCEPTOR = 0x0000_0000_0002_0000;

        /// ilabels unique leaving each state.
        const I_DETERMINISTIC = 0x0000_0000_0004_0000;
        /// ilabels not unique leaving some state.
        const NOT_I_DETERMINISTIC = 0x0000_0000_0008_0000;

        /// olabels unique leaving each state.
        const O_DETERMINISTIC = 0x0000_0000_0010_0000;
        /// olabels not unique leaving some state.
        const NOT_O_DETERMINISTIC = 0x0000_0000_0020_0000;

        /// FST has input/output epsilons.
        const EPSILONS = 0x0000_0000_0040_0000;
        /// FST has no input/output epsilons.
        const NO_EPSILONS = 0x0000_0000_0080_0000;

        /// FST has input epsilons.
        const I_EPSILONS = 0x0000_0000_0100_0000;
        /// FST has no input epsilons.
        const NO_I_EPSILONS = 0x0000_0000_0200_0000;

        /// FST has output epsilons.
        const O_EPSILONS = 0x0000_0000_0400_0000;
        /// FST has no output epsilons.
        const NO_O_EPSILONS = 0x0000_0000_0800_0000;

        /// ilabels sorted wrt < for each state.
        const I_LABEL_SORTED = 0x0000_0000_1000_0000;
        /// ilabels not sorted wrt < for some state.
        const NOT_I_LABEL_SORTED = 0x0000_0000_2000_0000;

        /// olabels sorted wrt < for each state.
        const O_LABEL_SORTED = 0x0000_0000_4000_0000;
        /// olabels not sorted wrt < for some state.
        const NOT_O_LABEL_SORTED = 0x0000_0000_8000_0000;

        /// Non-trivial transition or final weights.
        const WEIGHTED = 0x0000_0001_0000_0000;
        /// Only trivial transition and final weights.
        const UNWEIGHTED = 0x0000_0002_0000_0000;

        /// FST has cycles.
        const CYCLIC = 0x0000_0004_0000_0000;
        /// FST has no cycles.
        const ACYCLIC = 0x0000_0008_0000_0000;

        /// FST has cycles containing the initial state.
        const INITIAL_CYCLIC = 0x0000_0010_0000_0000;
        /// FST has no cycles containing the initial state.
        const INITIAL_ACYCLIC = 0x0000_0020_0000_0000;

        /// FST is topologically sorted.
        const TOP_SORTED = 0x0000_0040_0000_0000;
        /// FST is not topologically sorted.
        const NOT_TOP_SORTED = 0x0000_0080_0000_0000;

        /// All states reachable from the initial state.
        const ACCESSIBLE = 0x0000_0100_0000_0000;
        /// Not all states reachable from the initial state.
        const NOT_ACCESSIBLE = 0x0000_0200_0000_0000;

        /// All states can reach a final state.
        const COACCESSIBLE = 0x0000_0400_0000_0000;
        /// Not all states can reach a final state.
        const NOT_COACCESSIBLE = 0x0000_0800_0000_0000;

        /// If NumStates() > 0, then state 0 is initial, state NumStates() - 1 is final,
        /// there is a transition from each non-final state i to state i + 1, and there
        /// are no other transitions.
        const STRING = 0x0000_1000_0000_0000;

        /// Not a string FST.
        const NOT_STRING = 0x0000_2000_0000_0000;

        /// FST has at least one weighted cycle.
        const WEIGHTED_CYCLES = 0x0000_4000_0000_0000;

        /// Only unweighted cycles.
        const UNWEIGHTED_CYCLES = 0x0000_8000_0000_0000;
    }

}

impl FstProperties {
    /// Properties of an empty machine.
    pub(crate) fn null_properties() -> FstProperties {
        FstProperties::ACCEPTOR
            | FstProperties::I_DETERMINISTIC
            | FstProperties::O_DETERMINISTIC
            | FstProperties::NO_EPSILONS
            | FstProperties::NO_I_EPSILONS
            | FstProperties::NO_O_EPSILONS
            | FstProperties::I_LABEL_SORTED
            | FstProperties::O_LABEL_SORTED
            | FstProperties::UNWEIGHTED
            | FstProperties::ACYCLIC
            | FstProperties::INITIAL_ACYCLIC
            | FstProperties::TOP_SORTED
            | FstProperties::ACCESSIBLE
            | FstProperties::COACCESSIBLE
            | FstProperties::STRING
            | FstProperties::UNWEIGHTED_CYCLES
    }

    /// Properties that are preserved when an FST is copied.
    pub(crate) fn copy_properties() -> FstProperties {
        FstProperties::ACCEPTOR
            | FstProperties::NOT_ACCEPTOR
            | FstProperties::I_DETERMINISTIC
            | FstProperties::NOT_I_DETERMINISTIC
            | FstProperties::O_DETERMINISTIC
            | FstProperties::NOT_O_DETERMINISTIC
            | FstProperties::EPSILONS
            | FstProperties::NO_EPSILONS
            | FstProperties::I_EPSILONS
            | FstProperties::NO_I_EPSILONS
            | FstProperties::O_EPSILONS
            | FstProperties::NO_O_EPSILONS
            | FstProperties::I_LABEL_SORTED
            | FstProperties::NOT_I_LABEL_SORTED
            | FstProperties::O_LABEL_SORTED
            | FstProperties::NOT_O_LABEL_SORTED
            | FstProperties::WEIGHTED
            | FstProperties::UNWEIGHTED
            | FstProperties::CYCLIC
            | FstProperties::ACYCLIC
            | FstProperties::INITIAL_CYCLIC
            | FstProperties::INITIAL_ACYCLIC
            | FstProperties::TOP_SORTED
            | FstProperties::NOT_TOP_SORTED
            | FstProperties::ACCESSIBLE
            | FstProperties::NOT_ACCESSIBLE
            | FstProperties::COACCESSIBLE
            | FstProperties::NOT_COACCESSIBLE
            | FstProperties::STRING
            | FstProperties::NOT_STRING
            | FstProperties::WEIGHTED_CYCLES
            | FstProperties::UNWEIGHTED_CYCLES
    }

    /// Properties that are preserved when an FST start state is set.
    pub(crate) fn set_start_properties() -> FstProperties {
        FstProperties::ACCEPTOR
            | FstProperties::NOT_ACCEPTOR
            | FstProperties::I_DETERMINISTIC
            | FstProperties::NOT_I_DETERMINISTIC
            | FstProperties::O_DETERMINISTIC
            | FstProperties::NOT_O_DETERMINISTIC
            | FstProperties::EPSILONS
            | FstProperties::NO_EPSILONS
            | FstProperties::I_EPSILONS
            | FstProperties::NO_I_EPSILONS
            | FstProperties::O_EPSILONS
            | FstProperties::NO_O_EPSILONS
            | FstProperties::I_LABEL_SORTED
            | FstProperties::NOT_I_LABEL_SORTED
            | FstProperties::O_LABEL_SORTED
            | FstProperties::NOT_O_LABEL_SORTED
            | FstProperties::WEIGHTED
            | FstProperties::UNWEIGHTED
            | FstProperties::CYCLIC
            | FstProperties::ACYCLIC
            | FstProperties::TOP_SORTED
            | FstProperties::NOT_TOP_SORTED
            | FstProperties::COACCESSIBLE
            | FstProperties::NOT_COACCESSIBLE
            | FstProperties::WEIGHTED_CYCLES
            | FstProperties::UNWEIGHTED_CYCLES
    }

    /// Properties that are preserved when an FST final weight is set.
    pub(crate) fn set_final_properties() -> FstProperties {
        FstProperties::ACCEPTOR
            | FstProperties::NOT_ACCEPTOR
            | FstProperties::I_DETERMINISTIC
            | FstProperties::NOT_I_DETERMINISTIC
            | FstProperties::O_DETERMINISTIC
            | FstProperties::NOT_O_DETERMINISTIC
            | FstProperties::EPSILONS
            | FstProperties::NO_EPSILONS
            | FstProperties::I_EPSILONS
            | FstProperties::NO_I_EPSILONS
            | FstProperties::O_EPSILONS
            | FstProperties::NO_O_EPSILONS
            | FstProperties::I_LABEL_SORTED
            | FstProperties::NOT_I_LABEL_SORTED
            | FstProperties::O_LABEL_SORTED
            | FstProperties::NOT_O_LABEL_SORTED
            | FstProperties::CYCLIC
            | FstProperties::ACYCLIC
            | FstProperties::INITIAL_CYCLIC
            | FstProperties::INITIAL_ACYCLIC
            | FstProperties::TOP_SORTED
            | FstProperties::NOT_TOP_SORTED
            | FstProperties::ACCESSIBLE
            | FstProperties::NOT_ACCESSIBLE
            | FstProperties::WEIGHTED_CYCLES
            | FstProperties::UNWEIGHTED_CYCLES
    }

    /// Properties that are preserved when an FST state is added.
    pub(crate) fn add_state_properties() -> FstProperties {
        FstProperties::ACCEPTOR
            | FstProperties::NOT_ACCEPTOR
            | FstProperties::I_DETERMINISTIC
            | FstProperties::NOT_I_DETERMINISTIC
            | FstProperties::O_DETERMINISTIC
            | FstProperties::NOT_O_DETERMINISTIC
            | FstProperties::EPSILONS
            | FstProperties::NO_EPSILONS
            | FstProperties::I_EPSILONS
            | FstProperties::NO_I_EPSILONS
            | FstProperties::O_EPSILONS
            | FstProperties::NO_O_EPSILONS
            | FstProperties::I_LABEL_SORTED
            | FstProperties::NOT_I_LABEL_SORTED
            | FstProperties::O_LABEL_SORTED
            | FstProperties::NOT_O_LABEL_SORTED
            | FstProperties::WEIGHTED
            | FstProperties::UNWEIGHTED
            | FstProperties::CYCLIC
            | FstProperties::ACYCLIC
            | FstProperties::INITIAL_CYCLIC
            | FstProperties::INITIAL_ACYCLIC
            | FstProperties::TOP_SORTED
            | FstProperties::NOT_TOP_SORTED
            | FstProperties::NOT_ACCESSIBLE
            | FstProperties::NOT_COACCESSIBLE
            | FstProperties::NOT_STRING
            | FstProperties::WEIGHTED_CYCLES
            | FstProperties::UNWEIGHTED_CYCLES
    }

    /// Properties that are preserved when an FST transition is added.
    pub(crate) fn add_arc_properties() -> FstProperties {
        FstProperties::NOT_ACCEPTOR
            | FstProperties::NOT_I_DETERMINISTIC
            | FstProperties::NOT_O_DETERMINISTIC
            | FstProperties::EPSILONS
            | FstProperties::I_EPSILONS
            | FstProperties::O_EPSILONS
            | FstProperties::NOT_I_LABEL_SORTED
            | FstProperties::NOT_O_LABEL_SORTED
            | FstProperties::WEIGHTED
            | FstProperties::CYCLIC
            | FstProperties::INITIAL_CYCLIC
            | FstProperties::NOT_TOP_SORTED
            | FstProperties::ACCESSIBLE
            | FstProperties::COACCESSIBLE
            | FstProperties::WEIGHTED_CYCLES
    }

    /// Properties that are preserved when an FST transition is set.
    pub(crate) fn set_arc_properties() -> FstProperties {
        FstProperties::empty()
    }

    /// Properties that are preserved when FST states are deleted.
    pub(crate) fn delete_states_properties() -> FstProperties {
        FstProperties::ACCEPTOR
            | FstProperties::I_DETERMINISTIC
            | FstProperties::O_DETERMINISTIC
            | FstProperties::NO_EPSILONS
            | FstProperties::NO_I_EPSILONS
            | FstProperties::NO_O_EPSILONS
            | FstProperties::I_LABEL_SORTED
            | FstProperties::O_LABEL_SORTED
            | FstProperties::UNWEIGHTED
            | FstProperties::ACYCLIC
            | FstProperties::INITIAL_ACYCLIC
            | FstProperties::TOP_SORTED
            | FstProperties::UNWEIGHTED_CYCLES
    }

    /// Properties that are preserved when FST trs are deleted.
    pub(crate) fn delete_arcs_properties() -> FstProperties {
        FstProperties::ACCEPTOR
            | FstProperties::I_DETERMINISTIC
            | FstProperties::O_DETERMINISTIC
            | FstProperties::NO_EPSILONS
            | FstProperties::NO_I_EPSILONS
            | FstProperties::NO_O_EPSILONS
            | FstProperties::I_LABEL_SORTED
            | FstProperties::O_LABEL_SORTED
            | FstProperties::UNWEIGHTED
            | FstProperties::ACYCLIC
            | FstProperties::INITIAL_ACYCLIC
            | FstProperties::TOP_SORTED
            | FstProperties::NOT_ACCESSIBLE
            | FstProperties::NOT_COACCESSIBLE
            | FstProperties::UNWEIGHTED_CYCLES
    }

    /// Properties that are preserved when an FST's states are reordered.
    pub(crate) fn statesort_properties() -> FstProperties {
        FstProperties::ACCEPTOR
            | FstProperties::NOT_ACCEPTOR
            | FstProperties::I_DETERMINISTIC
            | FstProperties::NOT_I_DETERMINISTIC
            | FstProperties::O_DETERMINISTIC
            | FstProperties::NOT_O_DETERMINISTIC
            | FstProperties::EPSILONS
            | FstProperties::NO_EPSILONS
            | FstProperties::I_EPSILONS
            | FstProperties::NO_I_EPSILONS
            | FstProperties::O_EPSILONS
            | FstProperties::NO_O_EPSILONS
            | FstProperties::I_LABEL_SORTED
            | FstProperties::NOT_I_LABEL_SORTED
            | FstProperties::O_LABEL_SORTED
            | FstProperties::NOT_O_LABEL_SORTED
            | FstProperties::WEIGHTED
            | FstProperties::UNWEIGHTED
            | FstProperties::CYCLIC
            | FstProperties::ACYCLIC
            | FstProperties::INITIAL_CYCLIC
            | FstProperties::INITIAL_ACYCLIC
            | FstProperties::ACCESSIBLE
            | FstProperties::NOT_ACCESSIBLE
            | FstProperties::COACCESSIBLE
            | FstProperties::NOT_COACCESSIBLE
            | FstProperties::WEIGHTED_CYCLES
            | FstProperties::UNWEIGHTED_CYCLES
    }

    /// Properties that are preserved when an FST's trs are reordered.
    pub(crate) fn arcsort_properties() -> FstProperties {
        FstProperties::ACCEPTOR
            | FstProperties::NOT_ACCEPTOR
            | FstProperties::I_DETERMINISTIC
            | FstProperties::NOT_I_DETERMINISTIC
            | FstProperties::O_DETERMINISTIC
            | FstProperties::NOT_O_DETERMINISTIC
            | FstProperties::EPSILONS
            | FstProperties::NO_EPSILONS
            | FstProperties::I_EPSILONS
            | FstProperties::NO_I_EPSILONS
            | FstProperties::O_EPSILONS
            | FstProperties::NO_O_EPSILONS
            | FstProperties::WEIGHTED
            | FstProperties::UNWEIGHTED
            | FstProperties::CYCLIC
            | FstProperties::ACYCLIC
            | FstProperties::INITIAL_CYCLIC
            | FstProperties::INITIAL_ACYCLIC
            | FstProperties::TOP_SORTED
            | FstProperties::NOT_TOP_SORTED
            | FstProperties::ACCESSIBLE
            | FstProperties::NOT_ACCESSIBLE
            | FstProperties::COACCESSIBLE
            | FstProperties::NOT_COACCESSIBLE
            | FstProperties::STRING
            | FstProperties::NOT_STRING
            | FstProperties::WEIGHTED_CYCLES
            | FstProperties::UNWEIGHTED_CYCLES
    }

    /// Properties that are preserved when an FST's input labels are changed.
    pub(crate) fn i_label_invariant_properties() -> FstProperties {
        FstProperties::O_DETERMINISTIC
            | FstProperties::NOT_O_DETERMINISTIC
            | FstProperties::O_EPSILONS
            | FstProperties::NO_O_EPSILONS
            | FstProperties::O_LABEL_SORTED
            | FstProperties::NOT_O_LABEL_SORTED
            | FstProperties::WEIGHTED
            | FstProperties::UNWEIGHTED
            | FstProperties::CYCLIC
            | FstProperties::ACYCLIC
            | FstProperties::INITIAL_CYCLIC
            | FstProperties::INITIAL_ACYCLIC
            | FstProperties::TOP_SORTED
            | FstProperties::NOT_TOP_SORTED
            | FstProperties::ACCESSIBLE
            | FstProperties::NOT_ACCESSIBLE
            | FstProperties::COACCESSIBLE
            | FstProperties::NOT_COACCESSIBLE
            | FstProperties::STRING
            | FstProperties::NOT_STRING
            | FstProperties::WEIGHTED_CYCLES
            | FstProperties::UNWEIGHTED_CYCLES
    }

    /// Properties that are preserved when an FST's output labels are changed.
    pub(crate) fn o_label_invariant_properties() -> FstProperties {
        FstProperties::I_DETERMINISTIC
            | FstProperties::NOT_I_DETERMINISTIC
            | FstProperties::I_EPSILONS
            | FstProperties::NO_I_EPSILONS
            | FstProperties::I_LABEL_SORTED
            | FstProperties::NOT_I_LABEL_SORTED
            | FstProperties::WEIGHTED
            | FstProperties::UNWEIGHTED
            | FstProperties::CYCLIC
            | FstProperties::ACYCLIC
            | FstProperties::INITIAL_CYCLIC
            | FstProperties::INITIAL_ACYCLIC
            | FstProperties::TOP_SORTED
            | FstProperties::NOT_TOP_SORTED
            | FstProperties::ACCESSIBLE
            | FstProperties::NOT_ACCESSIBLE
            | FstProperties::COACCESSIBLE
            | FstProperties::NOT_COACCESSIBLE
            | FstProperties::STRING
            | FstProperties::NOT_STRING
            | FstProperties::WEIGHTED_CYCLES
            | FstProperties::UNWEIGHTED_CYCLES
    }

    /// Properties that are preserved when an FST's weights are changed. This
    /// assumes that the set of states that are non-final is not changed.
    pub(crate) fn weight_invariant_properties() -> FstProperties {
        FstProperties::ACCEPTOR
            | FstProperties::NOT_ACCEPTOR
            | FstProperties::I_DETERMINISTIC
            | FstProperties::NOT_I_DETERMINISTIC
            | FstProperties::O_DETERMINISTIC
            | FstProperties::NOT_O_DETERMINISTIC
            | FstProperties::EPSILONS
            | FstProperties::NO_EPSILONS
            | FstProperties::I_EPSILONS
            | FstProperties::NO_I_EPSILONS
            | FstProperties::O_EPSILONS
            | FstProperties::NO_O_EPSILONS
            | FstProperties::I_LABEL_SORTED
            | FstProperties::NOT_I_LABEL_SORTED
            | FstProperties::O_LABEL_SORTED
            | FstProperties::NOT_O_LABEL_SORTED
            | FstProperties::CYCLIC
            | FstProperties::ACYCLIC
            | FstProperties::INITIAL_CYCLIC
            | FstProperties::INITIAL_ACYCLIC
            | FstProperties::TOP_SORTED
            | FstProperties::NOT_TOP_SORTED
            | FstProperties::ACCESSIBLE
            | FstProperties::NOT_ACCESSIBLE
            | FstProperties::COACCESSIBLE
            | FstProperties::NOT_COACCESSIBLE
            | FstProperties::STRING
            | FstProperties::NOT_STRING
    }

    /// Properties that are preserved when a superfinal state is added and an FST's
    /// final weights are directed to it via new transitions.
    pub(crate) fn add_super_final_properties() -> FstProperties {
        FstProperties::NOT_ACCEPTOR
            | FstProperties::NOT_I_DETERMINISTIC
            | FstProperties::NOT_O_DETERMINISTIC
            | FstProperties::EPSILONS
            | FstProperties::I_EPSILONS
            | FstProperties::O_EPSILONS
            | FstProperties::NOT_I_LABEL_SORTED
            | FstProperties::NOT_O_LABEL_SORTED
            | FstProperties::WEIGHTED
            | FstProperties::UNWEIGHTED
            | FstProperties::CYCLIC
            | FstProperties::ACYCLIC
            | FstProperties::INITIAL_CYCLIC
            | FstProperties::INITIAL_ACYCLIC
            | FstProperties::NOT_TOP_SORTED
            | FstProperties::NOT_ACCESSIBLE
            | FstProperties::COACCESSIBLE
            | FstProperties::NOT_COACCESSIBLE
            | FstProperties::NOT_STRING
            | FstProperties::WEIGHTED_CYCLES
            | FstProperties::UNWEIGHTED_CYCLES
    }

    /// Properties that are preserved when a superfinal state is removed and the
    /// epsilon transitions directed to it are made final weights.
    pub(crate) fn rm_super_final_properties() -> FstProperties {
        FstProperties::ACCEPTOR
            | FstProperties::NOT_ACCEPTOR
            | FstProperties::I_DETERMINISTIC
            | FstProperties::O_DETERMINISTIC
            | FstProperties::NO_EPSILONS
            | FstProperties::NO_I_EPSILONS
            | FstProperties::NO_O_EPSILONS
            | FstProperties::I_LABEL_SORTED
            | FstProperties::O_LABEL_SORTED
            | FstProperties::WEIGHTED
            | FstProperties::UNWEIGHTED
            | FstProperties::CYCLIC
            | FstProperties::ACYCLIC
            | FstProperties::INITIAL_CYCLIC
            | FstProperties::INITIAL_ACYCLIC
            | FstProperties::TOP_SORTED
            | FstProperties::ACCESSIBLE
            | FstProperties::COACCESSIBLE
            | FstProperties::NOT_COACCESSIBLE
            | FstProperties::STRING
            | FstProperties::WEIGHTED_CYCLES
            | FstProperties::UNWEIGHTED_CYCLES
    }

    // Binary properties are not stored here so should be useless.
    pub(crate) fn binary_properties() -> FstProperties {
        FstProperties::from_bits_truncate(0x0000_0000_0000_0007)
    }
    pub(crate) fn trinary_properties() -> FstProperties {
        FstProperties::from_bits_truncate(0x0000_ffff_ffff_0000)
    }

    pub(crate) fn pos_trinary_properties() -> FstProperties {
        FstProperties::trinary_properties()
            & FstProperties::from_bits_truncate(0x5555_5555_5555_5555)
    }
    pub(crate) fn neg_trinary_properties() -> FstProperties {
        FstProperties::trinary_properties()
            & FstProperties::from_bits_truncate(0xaaaa_aaaa_aaaa_aaaa)
    }

    pub(crate) fn all_properties() -> FstProperties {
        FstProperties::binary_properties() | FstProperties::trinary_properties()
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
