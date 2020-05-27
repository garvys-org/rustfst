use crate::algorithms::closure::ClosureType;
use crate::algorithms::ProjectType;
use crate::fst_properties::FstProperties;
use crate::semirings::Semiring;
use crate::Tr;
use crate::{StateId, EPS_LABEL};

pub fn set_start_properties(inprops: FstProperties) -> FstProperties {
    let mut outprops = inprops & FstProperties::set_start_properties();
    if inprops.contains(FstProperties::ACYCLIC) {
        outprops |= FstProperties::INITIAL_ACYCLIC;
    }
    outprops
}

pub fn set_final_properties<W: Semiring>(
    inprops: FstProperties,
    old_weight: Option<&W>,
    new_weight: Option<&W>,
) -> FstProperties {
    let mut outprops = inprops;
    if let Some(w) = old_weight {
        if !w.is_zero() && !w.is_one() {
            outprops &= !FstProperties::WEIGHTED;
        }
    }

    if let Some(w) = new_weight {
        if !w.is_zero() && !w.is_one() {
            outprops |= FstProperties::WEIGHTED;
            outprops &= !FstProperties::UNWEIGHTED;
        }
    }

    outprops &=
        FstProperties::set_final_properties() | FstProperties::WEIGHTED | FstProperties::UNWEIGHTED;
    outprops
}

pub fn add_state_properties(inprops: FstProperties) -> FstProperties {
    inprops & FstProperties::add_state_properties()
}

pub fn add_tr_properties<W: Semiring>(
    inprops: FstProperties,
    state: StateId,
    tr: &Tr<W>,
    prev_tr: Option<&Tr<W>>,
) -> FstProperties {
    let mut outprops = inprops;

    if tr.ilabel != tr.olabel {
        outprops |= FstProperties::NOT_ACCEPTOR;
        outprops &= !FstProperties::ACCEPTOR;
    }
    if tr.ilabel == EPS_LABEL {
        outprops |= FstProperties::I_EPSILONS;
        outprops &= !FstProperties::NO_I_EPSILONS;
        if tr.olabel == EPS_LABEL {
            outprops |= FstProperties::EPSILONS;
            outprops &= !FstProperties::NO_EPSILONS;
        }
    }
    if tr.olabel == EPS_LABEL {
        outprops |= FstProperties::O_EPSILONS;
        outprops &= !FstProperties::NO_O_EPSILONS;
    }
    if let Some(prev_tr) = prev_tr {
        if prev_tr.ilabel > tr.ilabel {
            outprops |= FstProperties::NOT_I_LABEL_SORTED;
            outprops &= !FstProperties::I_LABEL_SORTED;
        }
        if prev_tr.olabel > tr.olabel {
            outprops |= FstProperties::NOT_O_LABEL_SORTED;
            outprops &= !FstProperties::O_LABEL_SORTED;
        }
    }
    if !tr.weight.is_zero() && !tr.weight.is_one() {
        outprops |= FstProperties::WEIGHTED;
        outprops &= !FstProperties::UNWEIGHTED;
    }
    if tr.nextstate <= state {
        outprops |= FstProperties::NOT_TOP_SORTED;
        outprops &= !FstProperties::TOP_SORTED;
    }
    outprops &= FstProperties::add_arc_properties()
        | FstProperties::ACCEPTOR
        | FstProperties::NO_EPSILONS
        | FstProperties::NO_I_EPSILONS
        | FstProperties::NO_O_EPSILONS
        | FstProperties::I_LABEL_SORTED
        | FstProperties::O_LABEL_SORTED
        | FstProperties::UNWEIGHTED
        | FstProperties::TOP_SORTED;

    if outprops.contains(FstProperties::TOP_SORTED) {
        outprops |= FstProperties::ACYCLIC | FstProperties::INITIAL_ACYCLIC;
    }

    outprops
}

pub fn delete_states_properties(inprops: FstProperties) -> FstProperties {
    inprops & FstProperties::delete_states_properties()
}

pub fn delete_all_states_properties() -> FstProperties {
    FstProperties::null_properties()
}

pub fn delete_trs_properties(inprops: FstProperties) -> FstProperties {
    inprops & FstProperties::delete_arcs_properties()
}

pub fn closure_properties(inprops: FstProperties, delayed: bool) -> FstProperties {
    let mut outprops =
        (FstProperties::ACCEPTOR | FstProperties::UNWEIGHTED | FstProperties::ACCESSIBLE) & inprops;
    if inprops.contains(FstProperties::UNWEIGHTED) {
        outprops |= FstProperties::UNWEIGHTED_CYCLES;
    }
    if !delayed {
        outprops |= (FstProperties::COACCESSIBLE
            | FstProperties::NOT_TOP_SORTED
            | FstProperties::NOT_STRING)
            & inprops;
    }
    if !delayed || inprops.contains(FstProperties::ACCESSIBLE) {
        outprops |= (FstProperties::NOT_ACCEPTOR
            | FstProperties::NOT_I_DETERMINISTIC
            | FstProperties::NOT_O_DETERMINISTIC
            | FstProperties::NOT_I_LABEL_SORTED
            | FstProperties::NOT_O_LABEL_SORTED
            | FstProperties::WEIGHTED
            | FstProperties::WEIGHTED_CYCLES
            | FstProperties::NOT_ACCESSIBLE
            | FstProperties::NOT_COACCESSIBLE)
            & inprops;
        if inprops.contains(FstProperties::WEIGHTED)
            && inprops.contains(FstProperties::ACCESSIBLE)
            && inprops.contains(FstProperties::COACCESSIBLE)
        {
            outprops |= FstProperties::WEIGHTED_CYCLES;
        }
    }
    outprops
}

pub fn complement_properties(_inprops: FstProperties) -> FstProperties {
    unimplemented!()
}

pub fn compose_properties(_inprops1: FstProperties, _inprops2: FstProperties) -> FstProperties {
    unimplemented!()
}

pub fn concat_properties(
    inprops1: FstProperties,
    inprops2: FstProperties,
    delayed: bool,
) -> FstProperties {
    let mut outprops = (FstProperties::ACCEPTOR
        | FstProperties::UNWEIGHTED
        | FstProperties::UNWEIGHTED_CYCLES
        | FstProperties::ACYCLIC)
        & inprops1
        & inprops2;
    let empty1 = delayed; // Can the first FST be the empty machine?
    let empty2 = delayed; // Can the second FST be the empty machine?
    if !delayed {
        outprops |= (FstProperties::NOT_TOP_SORTED | FstProperties::NOT_STRING) & inprops1;
        outprops |= (FstProperties::NOT_TOP_SORTED | FstProperties::NOT_STRING) & inprops2;
    }
    if !empty1 {
        outprops |= (FstProperties::INITIAL_ACYCLIC | FstProperties::INITIAL_CYCLIC) & inprops1;
    }
    if !delayed || inprops1.contains(FstProperties::ACCESSIBLE) {
        outprops |= (FstProperties::NOT_ACCEPTOR
            | FstProperties::NOT_I_DETERMINISTIC
            | FstProperties::NOT_O_DETERMINISTIC
            | FstProperties::EPSILONS
            | FstProperties::I_EPSILONS
            | FstProperties::O_EPSILONS
            | FstProperties::NOT_I_LABEL_SORTED
            | FstProperties::NOT_O_LABEL_SORTED
            | FstProperties::WEIGHTED
            | FstProperties::WEIGHTED_CYCLES
            | FstProperties::CYCLIC
            | FstProperties::NOT_ACCESSIBLE
            | FstProperties::NOT_COACCESSIBLE)
            & inprops1;
    }
    if (inprops1.contains(FstProperties::ACCESSIBLE | FstProperties::COACCESSIBLE)) && !empty1 {
        outprops |= FstProperties::ACCESSIBLE & inprops2;
        if !empty2 {
            outprops |= FstProperties::COACCESSIBLE & inprops2;
        }
        if !delayed || inprops2.contains(FstProperties::ACCESSIBLE) {
            outprops |= (FstProperties::NOT_ACCEPTOR
                | FstProperties::NOT_I_DETERMINISTIC
                | FstProperties::NOT_O_DETERMINISTIC
                | FstProperties::EPSILONS
                | FstProperties::I_EPSILONS
                | FstProperties::O_EPSILONS
                | FstProperties::NOT_I_LABEL_SORTED
                | FstProperties::NOT_O_LABEL_SORTED
                | FstProperties::WEIGHTED
                | FstProperties::WEIGHTED_CYCLES
                | FstProperties::CYCLIC
                | FstProperties::NOT_ACCESSIBLE
                | FstProperties::NOT_COACCESSIBLE)
                & inprops2;
        }
    }
    outprops
}

pub fn determinize_properties(_inprops: FstProperties) -> FstProperties {
    unimplemented!()
}

pub fn factor_weight_properties(_inprops: FstProperties) -> FstProperties {
    unimplemented!()
}

pub fn invert_properties(inprops: FstProperties) -> FstProperties {
    let mut outprops = (FstProperties::ACCEPTOR
        | FstProperties::NOT_ACCEPTOR
        | FstProperties::EPSILONS
        | FstProperties::NO_EPSILONS
        | FstProperties::WEIGHTED
        | FstProperties::UNWEIGHTED
        | FstProperties::WEIGHTED_CYCLES
        | FstProperties::UNWEIGHTED_CYCLES
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
        | FstProperties::NOT_STRING)
        & inprops;
    if inprops.contains(FstProperties::I_DETERMINISTIC) {
        outprops |= FstProperties::O_DETERMINISTIC;
    }
    if inprops.contains(FstProperties::NOT_I_DETERMINISTIC) {
        outprops |= FstProperties::NOT_O_DETERMINISTIC;
    }
    if inprops.contains(FstProperties::O_DETERMINISTIC) {
        outprops |= FstProperties::I_DETERMINISTIC;
    }
    if inprops.contains(FstProperties::NOT_O_DETERMINISTIC) {
        outprops |= FstProperties::NOT_I_DETERMINISTIC;
    }

    if inprops.contains(FstProperties::I_EPSILONS) {
        outprops |= FstProperties::O_EPSILONS;
    }
    if inprops.contains(FstProperties::NO_I_EPSILONS) {
        outprops |= FstProperties::NO_O_EPSILONS;
    }
    if inprops.contains(FstProperties::O_EPSILONS) {
        outprops |= FstProperties::I_EPSILONS;
    }
    if inprops.contains(FstProperties::NO_O_EPSILONS) {
        outprops |= FstProperties::NO_I_EPSILONS;
    }

    if inprops.contains(FstProperties::I_LABEL_SORTED) {
        outprops |= FstProperties::O_LABEL_SORTED;
    }
    if inprops.contains(FstProperties::NOT_I_LABEL_SORTED) {
        outprops |= FstProperties::NOT_O_LABEL_SORTED;
    }
    if inprops.contains(FstProperties::O_LABEL_SORTED) {
        outprops |= FstProperties::I_LABEL_SORTED;
    }
    if inprops.contains(FstProperties::NOT_O_LABEL_SORTED) {
        outprops |= FstProperties::NOT_I_LABEL_SORTED;
    }
    outprops
}

pub fn project_properties(inprops: FstProperties, project_type: ProjectType) -> FstProperties {
    let mut outprops = FstProperties::ACCEPTOR;
    outprops |= (FstProperties::WEIGHTED
        | FstProperties::UNWEIGHTED
        | FstProperties::WEIGHTED_CYCLES
        | FstProperties::UNWEIGHTED_CYCLES
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
        | FstProperties::NOT_STRING)
        & inprops;
    match project_type {
        ProjectType::ProjectInput => {
            outprops |= (FstProperties::I_DETERMINISTIC
                | FstProperties::NOT_I_DETERMINISTIC
                | FstProperties::I_EPSILONS
                | FstProperties::NO_I_EPSILONS
                | FstProperties::I_LABEL_SORTED
                | FstProperties::NOT_I_LABEL_SORTED)
                & inprops;

            if inprops.contains(FstProperties::I_DETERMINISTIC) {
                outprops |= FstProperties::O_DETERMINISTIC;
            }
            if inprops.contains(FstProperties::NOT_I_DETERMINISTIC) {
                outprops |= FstProperties::NOT_O_DETERMINISTIC;
            }

            if inprops.contains(FstProperties::I_EPSILONS) {
                outprops |= FstProperties::O_EPSILONS | FstProperties::EPSILONS;
            }
            if inprops.contains(FstProperties::NO_I_EPSILONS) {
                outprops |= FstProperties::NO_O_EPSILONS | FstProperties::NO_EPSILONS;
            }

            if inprops.contains(FstProperties::I_LABEL_SORTED) {
                outprops |= FstProperties::O_LABEL_SORTED;
            }
            if inprops.contains(FstProperties::NOT_I_LABEL_SORTED) {
                outprops |= FstProperties::NOT_O_LABEL_SORTED;
            }
        }
        ProjectType::ProjectOutput => {
            outprops |= (FstProperties::O_DETERMINISTIC
                | FstProperties::NOT_O_DETERMINISTIC
                | FstProperties::O_EPSILONS
                | FstProperties::NO_O_EPSILONS
                | FstProperties::O_LABEL_SORTED
                | FstProperties::NOT_O_LABEL_SORTED)
                & inprops;

            if inprops.contains(FstProperties::O_DETERMINISTIC) {
                outprops |= FstProperties::I_DETERMINISTIC;
            }
            if inprops.contains(FstProperties::NOT_O_DETERMINISTIC) {
                outprops |= FstProperties::NOT_I_DETERMINISTIC;
            }

            if inprops.contains(FstProperties::O_EPSILONS) {
                outprops |= FstProperties::I_EPSILONS | FstProperties::EPSILONS;
            }
            if inprops.contains(FstProperties::NO_O_EPSILONS) {
                outprops |= FstProperties::NO_I_EPSILONS | FstProperties::NO_EPSILONS;
            }

            if inprops.contains(FstProperties::O_LABEL_SORTED) {
                outprops |= FstProperties::I_LABEL_SORTED;
            }
            if inprops.contains(FstProperties::NOT_O_LABEL_SORTED) {
                outprops |= FstProperties::NOT_I_LABEL_SORTED;
            }
        }
    };
    outprops
}

pub fn rand_gen_propertoes(_inprops: FstProperties) -> FstProperties {
    unimplemented!()
}

pub fn relabel_properties(_inprops: FstProperties) -> FstProperties {
    unimplemented!()
}

pub fn replace_properties(_inprops: FstProperties) -> FstProperties {
    unimplemented!()
}

pub fn reverse_properties(_inprops: FstProperties) -> FstProperties {
    unimplemented!()
}

pub fn reweight_properties(_inprops: FstProperties) -> FstProperties {
    unimplemented!()
}

pub fn rmepsilon_properties(_inprops: FstProperties) -> FstProperties {
    unimplemented!()
}

pub fn shortest_path_properties(_inprops: FstProperties) -> FstProperties {
    unimplemented!()
}

pub fn synchronization_properties(_inprops: FstProperties) -> FstProperties {
    unimplemented!()
}

pub fn union_properties(
    inprops1: FstProperties,
    inprops2: FstProperties,
    delayed: bool,
) -> FstProperties {
    let mut outprops = (FstProperties::ACCEPTOR
        | FstProperties::UNWEIGHTED
        | FstProperties::UNWEIGHTED_CYCLES
        | FstProperties::ACYCLIC
        | FstProperties::ACCESSIBLE)
        & inprops1
        & inprops2;
    outprops |= FstProperties::INITIAL_ACYCLIC;
    let empty1 = delayed; // Can the first FST be the empty machine?
    let empty2 = delayed; // Can the second FST be the empty machine?
    if !delayed {
        outprops |= FstProperties::NOT_TOP_SORTED & inprops1;
        outprops |= FstProperties::NOT_TOP_SORTED & inprops2;
    }
    if !empty1 && !empty2 {
        outprops |= FstProperties::EPSILONS | FstProperties::I_EPSILONS | FstProperties::O_EPSILONS;
        outprops |= FstProperties::COACCESSIBLE & inprops1 & inprops2;
    }
    // Note FstProperties::NOT_COACCESSIBLE does not hold because of FstProperties::INITIAL_ACYCLIC option.
    if !delayed || inprops1.contains(FstProperties::ACCESSIBLE) {
        outprops |= (FstProperties::NOT_ACCEPTOR
            | FstProperties::NOT_I_DETERMINISTIC
            | FstProperties::NOT_O_DETERMINISTIC
            | FstProperties::EPSILONS
            | FstProperties::I_EPSILONS
            | FstProperties::O_EPSILONS
            | FstProperties::NOT_I_LABEL_SORTED
            | FstProperties::NOT_O_LABEL_SORTED
            | FstProperties::WEIGHTED
            | FstProperties::WEIGHTED_CYCLES
            | FstProperties::CYCLIC
            | FstProperties::NOT_ACCESSIBLE)
            & inprops1;
    }
    if !delayed || inprops2.contains(FstProperties::ACCESSIBLE) {
        outprops |= (FstProperties::NOT_ACCEPTOR
            | FstProperties::NOT_I_DETERMINISTIC
            | FstProperties::NOT_O_DETERMINISTIC
            | FstProperties::EPSILONS
            | FstProperties::I_EPSILONS
            | FstProperties::O_EPSILONS
            | FstProperties::NOT_I_LABEL_SORTED
            | FstProperties::NOT_O_LABEL_SORTED
            | FstProperties::WEIGHTED
            | FstProperties::WEIGHTED_CYCLES
            | FstProperties::CYCLIC
            | FstProperties::NOT_ACCESSIBLE
            | FstProperties::NOT_COACCESSIBLE)
            & inprops2;
    }
    outprops
}
