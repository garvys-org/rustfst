use std::path::PathBuf;

use crate::arc::Arc;
use crate::fst_impls::vector::vector_fst::VectorFstState;
use crate::fst_impls::VectorFst;
use crate::parsers::text::{FinalState, ParsedTextFst, Transition};
use crate::semirings::{ProbabilityWeight, Semiring};
use crate::test_data::text_fst::TextParserTest;

#[cfg(test)]
pub(crate) fn text_fst_001() -> TextParserTest {
    let mut transitions = vec![];
    transitions.push(Transition::new(0, 32, 33, Some(0.5), 1));
    transitions.push(Transition::new(1, 44, 45, Some(0.13), 2));
    transitions.push(Transition::new(2, 17, 18, Some(0.25), 3));
    transitions.push(Transition::new(3, 45, 47, Some(0.41), 4));
    transitions.push(Transition::new(4, 5, 6, Some(0.31), 5));

    let mut final_states = vec![];
    final_states.push(FinalState::new(5, Some(0.67)));

    let s0 = VectorFstState {
        final_weight: None,
        arcs: vec![Arc::new(32, 33, ProbabilityWeight::new(0.5), 1)],
    };

    let s1 = VectorFstState {
        final_weight: None,
        arcs: vec![Arc::new(44, 45, ProbabilityWeight::new(0.13), 2)],
    };

    let s2 = VectorFstState {
        final_weight: None,
        arcs: vec![Arc::new(17, 18, ProbabilityWeight::new(0.25), 3)],
    };

    let s3 = VectorFstState {
        final_weight: None,
        arcs: vec![Arc::new(45, 47, ProbabilityWeight::new(0.41), 4)],
    };

    let s4 = VectorFstState {
        final_weight: None,
        arcs: vec![Arc::new(5, 6, ProbabilityWeight::new(0.31), 5)],
    };

    let s5 = VectorFstState {
        final_weight: Some(ProbabilityWeight::new(0.67)),
        arcs: vec![],
    };

    let vector_fst = VectorFst {
        start_state: Some(0),
        states: vec![s0, s1, s2, s3, s4, s5],
    };

    TextParserTest {
        name: "test_fst_001".to_string(),
        path: rel_to_abs_path!("text_fst_001.txt"),
        parsed_text_fst: ParsedTextFst {
            transitions,
            final_states,
        },
        vector_fst,
    }
}
