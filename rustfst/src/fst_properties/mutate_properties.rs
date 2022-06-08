use crate::algorithms::ProjectType;
use crate::fst_properties::FstProperties;
use crate::semirings::Semiring;
use crate::{Label, Tr};
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

pub fn compose_properties(inprops1: FstProperties, inprops2: FstProperties) -> FstProperties {
    let mut outprops = FstProperties::empty();
    if inprops1.contains(FstProperties::ACCEPTOR) && inprops2.contains(FstProperties::ACCEPTOR) {
        outprops |= FstProperties::ACCEPTOR | FstProperties::ACCESSIBLE;
        outprops |= (FstProperties::NO_EPSILONS
            | FstProperties::NO_I_EPSILONS
            | FstProperties::NO_O_EPSILONS
            | FstProperties::ACYCLIC
            | FstProperties::INITIAL_ACYCLIC)
            & inprops1
            & inprops2;
        if inprops1.contains(FstProperties::NO_I_EPSILONS)
            && inprops2.contains(FstProperties::NO_I_EPSILONS)
        {
            outprops |= (FstProperties::I_DETERMINISTIC | FstProperties::O_DETERMINISTIC)
                & inprops1
                & inprops2;
        }
    } else {
        outprops |= FstProperties::ACCESSIBLE;
        outprops |= (FstProperties::ACCEPTOR
            | FstProperties::NO_I_EPSILONS
            | FstProperties::ACYCLIC
            | FstProperties::INITIAL_ACYCLIC)
            & inprops1
            & inprops2;
        if inprops1.contains(FstProperties::NO_I_EPSILONS)
            && inprops2.contains(FstProperties::NO_I_EPSILONS)
        {
            outprops |= FstProperties::I_DETERMINISTIC & inprops1 & inprops2;
        }
    }
    outprops
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

pub fn determinize_properties(
    inprops: FstProperties,
    has_subsequential_label: bool,
    distinct_psubsequential_labels: bool,
) -> FstProperties {
    let mut outprops = FstProperties::ACCESSIBLE;
    if inprops.contains(FstProperties::ACCEPTOR)
        || (inprops.contains(FstProperties::NO_I_EPSILONS) && distinct_psubsequential_labels)
        || (has_subsequential_label && distinct_psubsequential_labels)
    {
        outprops |= FstProperties::I_DETERMINISTIC;
    }
    outprops |= (FstProperties::ACCEPTOR
        | FstProperties::ACYCLIC
        | FstProperties::INITIAL_ACYCLIC
        | FstProperties::COACCESSIBLE
        | FstProperties::STRING)
        & inprops;
    if inprops.contains(FstProperties::NO_I_EPSILONS) && distinct_psubsequential_labels {
        outprops |= FstProperties::NO_EPSILONS & inprops;
    }
    if inprops.contains(FstProperties::ACCESSIBLE) {
        outprops |= (FstProperties::I_EPSILONS | FstProperties::O_EPSILONS | FstProperties::CYCLIC)
            & inprops;
    }
    if inprops.contains(FstProperties::ACCEPTOR) {
        outprops |= (FstProperties::NO_I_EPSILONS | FstProperties::NO_O_EPSILONS) & inprops;
    }
    if inprops.contains(FstProperties::NO_I_EPSILONS) && has_subsequential_label {
        outprops |= FstProperties::NO_I_EPSILONS;
    }
    outprops
}

pub fn factor_weight_properties(inprops: FstProperties) -> FstProperties {
    let mut outprops = (FstProperties::ACCEPTOR
        | FstProperties::ACYCLIC
        | FstProperties::ACCESSIBLE
        | FstProperties::COACCESSIBLE)
        & inprops;
    if inprops.contains(FstProperties::ACCESSIBLE) {
        outprops |= (FstProperties::NOT_ACCEPTOR
            | FstProperties::NOT_I_DETERMINISTIC
            | FstProperties::NOT_O_DETERMINISTIC
            | FstProperties::EPSILONS
            | FstProperties::I_EPSILONS
            | FstProperties::O_EPSILONS
            | FstProperties::CYCLIC
            | FstProperties::NOT_I_LABEL_SORTED
            | FstProperties::NOT_O_LABEL_SORTED)
            & inprops;
    }
    outprops
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

pub fn rand_gen_properties(inprops: FstProperties, weighted: bool) -> FstProperties {
    let mut outprops = FstProperties::ACYCLIC
        | FstProperties::INITIAL_ACYCLIC
        | FstProperties::ACCESSIBLE
        | FstProperties::UNWEIGHTED_CYCLES;
    if weighted {
        outprops |= FstProperties::TOP_SORTED;
        outprops |= (FstProperties::ACCEPTOR
            | FstProperties::NO_EPSILONS
            | FstProperties::NO_I_EPSILONS
            | FstProperties::NO_O_EPSILONS
            | FstProperties::I_DETERMINISTIC
            | FstProperties::O_DETERMINISTIC
            | FstProperties::I_LABEL_SORTED
            | FstProperties::O_LABEL_SORTED)
            & inprops;
    } else {
        outprops |= FstProperties::UNWEIGHTED;
        outprops |= (FstProperties::ACCEPTOR
            | FstProperties::I_LABEL_SORTED
            | FstProperties::O_LABEL_SORTED)
            & inprops;
    }
    outprops
}

pub fn relabel_properties(inprops: FstProperties) -> FstProperties {
    let outprops = FstProperties::WEIGHTED
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
        | FstProperties::NOT_STRING;
    outprops & inprops
}

#[allow(clippy::too_many_arguments)]
pub fn replace_properties(
    inprops: &[FstProperties],
    root: Label,
    epsilon_on_call: bool,
    epsilon_on_return: bool,
    out_epsilon_on_call: bool,
    out_epsilon_on_return: bool,
    replace_transducer: bool,
    no_empty_fsts: bool,
    all_ilabel_sorted: bool,
    all_olabel_sorted: bool,
    all_negative_or_dense: bool,
) -> FstProperties {
    if inprops.is_empty() {
        return FstProperties::null_properties();
    }
    let mut outprops = FstProperties::empty();
    let mut access_props = if no_empty_fsts {
        FstProperties::ACCESSIBLE | FstProperties::COACCESSIBLE
    } else {
        FstProperties::empty()
    };

    for inprop in inprops {
        access_props &= *inprop & (FstProperties::ACCESSIBLE | FstProperties::COACCESSIBLE);
    }

    if access_props == (FstProperties::ACCESSIBLE | FstProperties::COACCESSIBLE) {
        outprops |= access_props;
        if inprops[root as usize].contains(FstProperties::INITIAL_CYCLIC) {
            outprops |= FstProperties::INITIAL_CYCLIC;
        }
        let mut props = FstProperties::empty();
        let mut string = true;
        for inprop in inprops {
            if replace_transducer {
                props |= FstProperties::NOT_ACCEPTOR & *inprop;
            }
            props |= (FstProperties::NOT_I_DETERMINISTIC
                | FstProperties::NOT_O_DETERMINISTIC
                | FstProperties::EPSILONS
                | FstProperties::I_EPSILONS
                | FstProperties::O_EPSILONS
                | FstProperties::WEIGHTED
                | FstProperties::WEIGHTED_CYCLES
                | FstProperties::CYCLIC
                | FstProperties::NOT_TOP_SORTED
                | FstProperties::NOT_STRING)
                & *inprop;
            if !inprop.contains(FstProperties::STRING) {
                string = false;
            }
        }
        outprops |= props;
        if string {
            outprops |= FstProperties::STRING;
        }
    }
    let mut acceptor = !replace_transducer;
    let mut ideterministic = !epsilon_on_call && epsilon_on_return;
    let mut no_iepsilons = !epsilon_on_call && !epsilon_on_return;
    let mut acyclic = true;
    let mut unweighted = true;
    for (i, inprop) in inprops.iter().enumerate() {
        if !inprop.contains(FstProperties::ACCEPTOR) {
            acceptor = false;
        }
        if !inprop.contains(FstProperties::I_DETERMINISTIC) {
            ideterministic = false;
        }
        if !inprop.contains(FstProperties::NO_I_EPSILONS) {
            no_iepsilons = false;
        }
        if !inprop.contains(FstProperties::ACYCLIC) {
            acyclic = false;
        }
        if !inprop.contains(FstProperties::UNWEIGHTED) {
            unweighted = false;
        }
        if i != root as usize && !inprop.contains(FstProperties::NO_I_EPSILONS) {
            ideterministic = false;
        }
    }
    if acceptor {
        outprops |= FstProperties::ACCEPTOR;
    }
    if ideterministic {
        outprops |= FstProperties::I_DETERMINISTIC;
    }
    if no_iepsilons {
        outprops |= FstProperties::NO_I_EPSILONS;
    }
    if acyclic {
        outprops |= FstProperties::ACYCLIC;
    }
    if unweighted {
        outprops |= FstProperties::UNWEIGHTED;
    }
    if inprops[root as usize].contains(FstProperties::INITIAL_ACYCLIC) {
        outprops |= FstProperties::INITIAL_ACYCLIC;
    }
    // We assume that all terminals are positive. The resulting ReplaceFst is
    // known to be FstProperties::I_LABEL_SORTED when: (1) all sub-FSTs are FstProperties::I_LABEL_SORTED, (2) the
    // input label of the return arc is epsilon, and (3) one of the 3 following
    // conditions is satisfied:
    //
    //  1. the input label of the call arc is not epsilon
    //  2. all non-terminals are negative, or
    //  3. all non-terninals are positive and form a dense range containing 1.
    if all_ilabel_sorted && epsilon_on_return && (!epsilon_on_call || all_negative_or_dense) {
        outprops |= FstProperties::I_LABEL_SORTED;
    }
    // Similarly, the resulting ReplaceFst is known to be FstProperties::O_LABEL_SORTED when: (1)
    // all sub-FSTs are FstProperties::O_LABEL_SORTED, (2) the output label of the return arc is
    // epsilon, and (3) one of the 3 following conditions is satisfied:
    //
    //  1. the output label of the call arc is not epsilon
    //  2. all non-terminals are negative, or
    //  3. all non-terninals are positive and form a dense range containing 1.
    if all_olabel_sorted && out_epsilon_on_return && (!out_epsilon_on_call || all_negative_or_dense)
    {
        outprops |= FstProperties::O_LABEL_SORTED;
    }
    outprops
}

pub fn reverse_properties(inprops: FstProperties, has_superinitial: bool) -> FstProperties {
    let mut outprops = (FstProperties::ACCEPTOR
        | FstProperties::NOT_ACCEPTOR
        | FstProperties::EPSILONS
        | FstProperties::I_EPSILONS
        | FstProperties::O_EPSILONS
        | FstProperties::UNWEIGHTED
        | FstProperties::CYCLIC
        | FstProperties::ACYCLIC
        | FstProperties::WEIGHTED_CYCLES
        | FstProperties::UNWEIGHTED_CYCLES)
        & inprops;
    if has_superinitial {
        outprops |= FstProperties::WEIGHTED & inprops;
    }
    outprops
}

pub fn reweight_properties(inprops: FstProperties) -> FstProperties {
    let mut outprops = inprops & FstProperties::weight_invariant_properties();
    outprops &= !FstProperties::COACCESSIBLE;
    outprops
}

pub fn rmepsilon_properties(inprops: FstProperties, delayed: bool) -> FstProperties {
    let mut outprops = FstProperties::NO_EPSILONS;
    outprops |= (FstProperties::ACCEPTOR | FstProperties::ACYCLIC | FstProperties::INITIAL_ACYCLIC)
        & inprops;
    if inprops.contains(FstProperties::ACCEPTOR) {
        outprops |= FstProperties::NO_I_EPSILONS | FstProperties::NO_O_EPSILONS;
    }
    if !delayed {
        outprops |= FstProperties::TOP_SORTED & inprops;
    }
    if !delayed || inprops.contains(FstProperties::ACCESSIBLE) {
        outprops |= FstProperties::NOT_ACCEPTOR & inprops;
    }
    outprops
}

pub fn shortest_path_properties(props: FstProperties, tree: bool) -> FstProperties {
    let mut outprops = props
        | FstProperties::ACYCLIC
        | FstProperties::INITIAL_ACYCLIC
        | FstProperties::ACCESSIBLE
        | FstProperties::UNWEIGHTED_CYCLES;
    if !tree {
        outprops |= FstProperties::COACCESSIBLE;
    }
    outprops
}

pub fn synchronization_properties(inprops: FstProperties) -> FstProperties {
    let mut outprops = (FstProperties::ACCEPTOR
        | FstProperties::ACYCLIC
        | FstProperties::ACCESSIBLE
        | FstProperties::COACCESSIBLE
        | FstProperties::UNWEIGHTED
        | FstProperties::UNWEIGHTED_CYCLES)
        & inprops;
    if inprops.contains(FstProperties::ACCESSIBLE) {
        outprops |= (FstProperties::CYCLIC
            | FstProperties::NOT_COACCESSIBLE
            | FstProperties::WEIGHTED
            | FstProperties::WEIGHTED_CYCLES)
            & inprops;
    }
    outprops
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
