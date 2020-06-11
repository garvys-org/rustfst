use std::collections::HashMap;

use anyhow::Result;

use crate::fst_properties::FstProperties;
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::semirings::WeightQuantize;
use crate::tests_openfst::FstTestData;

pub fn parse_fst_properties(mapping: &HashMap<String, bool>) -> FstProperties {
    let mut props = FstProperties::empty();

    // 1
    if mapping["acceptor"] {
        props |= FstProperties::ACCEPTOR
    }
    // 2
    if mapping["not_acceptor"] {
        props |= FstProperties::NOT_ACCEPTOR
    }
    // 3
    if mapping["i_deterministic"] {
        props |= FstProperties::I_DETERMINISTIC
    }
    // 4
    if mapping["not_i_deterministic"] {
        props |= FstProperties::NOT_I_DETERMINISTIC
    }
    // 5
    if mapping["o_deterministic"] {
        props |= FstProperties::O_DETERMINISTIC
    }
    // 6
    if mapping["not_o_deterministic"] {
        props |= FstProperties::NOT_O_DETERMINISTIC
    }
    // 7
    if mapping["epsilons"] {
        props |= FstProperties::EPSILONS
    }
    // 8
    if mapping["no_epsilons"] {
        props |= FstProperties::NO_EPSILONS
    }
    // 9
    if mapping["i_epsilons"] {
        props |= FstProperties::I_EPSILONS
    }
    // 10
    if mapping["no_i_epsilons"] {
        props |= FstProperties::NO_I_EPSILONS
    }
    // 11
    if mapping["o_epsilons"] {
        props |= FstProperties::O_EPSILONS
    }
    // 12
    if mapping["no_o_epsilons"] {
        props |= FstProperties::NO_O_EPSILONS
    }
    // 13
    if mapping["i_label_sorted"] {
        props |= FstProperties::I_LABEL_SORTED
    }
    // 14
    if mapping["not_i_label_sorted"] {
        props |= FstProperties::NOT_I_LABEL_SORTED
    }
    // 15
    if mapping["o_label_sorted"] {
        props |= FstProperties::O_LABEL_SORTED
    }
    // 16
    if mapping["not_o_label_sorted"] {
        props |= FstProperties::NOT_O_LABEL_SORTED
    }
    // 17
    if mapping["weighted"] {
        props |= FstProperties::WEIGHTED
    }
    // 18
    if mapping["unweighted"] {
        props |= FstProperties::UNWEIGHTED
    }
    // 19
    if mapping["cyclic"] {
        props |= FstProperties::CYCLIC
    }
    // 20
    if mapping["acyclic"] {
        props |= FstProperties::ACYCLIC
    }
    // 21
    if mapping["initial_cyclic"] {
        props |= FstProperties::INITIAL_CYCLIC
    }
    // 22
    if mapping["initial_acyclic"] {
        props |= FstProperties::INITIAL_ACYCLIC
    }
    // 23
    if mapping["top_sorted"] {
        props |= FstProperties::TOP_SORTED
    }
    // 24
    if mapping["not_top_sorted"] {
        props |= FstProperties::NOT_TOP_SORTED
    }
    // 25
    if mapping["accessible"] {
        props |= FstProperties::ACCESSIBLE
    }
    // 26
    if mapping["not_accessible"] {
        props |= FstProperties::NOT_ACCESSIBLE
    }
    // 27
    if mapping["coaccessible"] {
        props |= FstProperties::COACCESSIBLE
    }
    // 28
    if mapping["not_coaccessible"] {
        props |= FstProperties::NOT_COACCESSIBLE
    }
    // 29
    if mapping["string"] {
        props |= FstProperties::STRING
    }
    // 30
    if mapping["not_string"] {
        props |= FstProperties::NOT_STRING
    }
    // 31
    if mapping["weighted_cycles"] {
        props |= FstProperties::WEIGHTED_CYCLES
    }
    // 32
    if mapping["unweighted_cycles"] {
        props |= FstProperties::UNWEIGHTED_CYCLES
    }

    props
}

pub fn test_fst_properties<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W>,
    W: SerializableSemiring + WeightQuantize,
{
    let ref_props = test_data.fst_properties;
    let props = test_data.raw.properties();

    assert_eq!(props, ref_props);

    Ok(())
}
