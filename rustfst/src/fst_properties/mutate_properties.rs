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
        if !w.is_one() {
            outprops &= !FstProperties::WEIGHTED;
        }
    }

    if let Some(w) = new_weight {
        if !w.is_one() {
            outprops |= FstProperties::WEIGHTED;
            outprops &= !FstProperties::UNWEIGHTED;
        }
    }

    outprops &= FstProperties::set_final_properties() | FstProperties::WEIGHTED | FstProperties::UNWEIGHTED;
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
        outprops &= FstProperties::TOP_SORTED;
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

pub fn closure_properties(_inprops: FstProperties, _star: bool) -> FstProperties {
    unimplemented!()
}

pub fn complement_properties(_inprops: FstProperties) -> FstProperties {
    unimplemented!()
}

pub fn compose_properties(_inprops1: FstProperties, _inprops2: FstProperties) -> FstProperties {
    unimplemented!()
}

pub fn concat_properties(_inprops1: FstProperties, _inprops2: FstProperties) -> FstProperties {
    unimplemented!()
}

pub fn determinize_properties(_inprops: FstProperties) -> FstProperties {
    unimplemented!()
}

pub fn factor_weight_properties(_inprops: FstProperties) -> FstProperties {
    unimplemented!()
}

pub fn invert_properties(_inprops: FstProperties) -> FstProperties {
    unimplemented!()
}

pub fn project_properties(_inprops: FstProperties, _project_type: ProjectType) -> FstProperties {
    unimplemented!()
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

pub fn union_properties(_inprops: FstProperties) -> FstProperties {
    unimplemented!()
}
