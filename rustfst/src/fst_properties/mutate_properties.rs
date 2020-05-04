use crate::algorithms::ProjectType;
use crate::fst_properties::FstProperties;
use crate::semirings::Semiring;
use crate::StateId;
use crate::Tr;

pub fn set_start_properties(inprops: FstProperties) -> FstProperties {
    let mut outprops = inprops & FstProperties::SET_START_PROPERTIES;
    if inprops.contains(FstProperties::ACYCLIC) {
        outprops |= FstProperties::INITIAL_ACYCLIC;
    }
    outprops
}

pub fn set_final_properties<W: Semiring>(
    _inprops: FstProperties,
    _old_weight: &W,
    _new_weight: &W,
) -> FstProperties {
    unimplemented!()
}

pub fn add_state_properties(inprops: FstProperties) -> FstProperties {
    inprops & FstProperties::ADD_STATE_PROPERTIES
}

pub fn add_tr_properties<W: Semiring>(
    _inprops: FstProperties,
    _state: StateId,
    _old_tr: Tr<W>,
    _new_tr: Tr<W>,
) -> FstProperties {
    unimplemented!()
}

pub fn delete_states_properties(inprops: FstProperties) -> FstProperties {
    inprops & FstProperties::DELETE_STATES_PROPERTIES
}

pub fn delete_all_states_properties(_inprops: FstProperties) -> FstProperties {
    unimplemented!()
}

pub fn delete_trs_properties(inprops: FstProperties) -> FstProperties {
    inprops & FstProperties::DELETE_ARCS_PROPERTIES
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
