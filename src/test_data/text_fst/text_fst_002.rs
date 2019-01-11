use std::path::PathBuf;

use crate::arc::Arc;
use crate::fst_impls::vector::vector_fst::VectorFstState;
use crate::fst_impls::VectorFst;
use crate::parsers::text_fst::{FinalState, ParsedTextFst, Transition};
use crate::semirings::{ProbabilityWeight, Semiring};
use crate::test_data::text_fst::TextParserTest;

#[cfg(test)]
pub(crate) fn text_fst_002() -> TextParserTest {
    let mut transitions = vec![];
    transitions.push(Transition::new(0, 32, 32, None, 1));
    transitions.push(Transition::new(1, 45, 45, None, 2));
    transitions.push(Transition::new(2, 18, 18, Some(0.25), 3));
    transitions.push(Transition::new(3, 45, 45, None, 4));
    transitions.push(Transition::new(4, 5, 5, Some(0.31), 5));

    let mut final_states = vec![];
    final_states.push(FinalState::new(5, None));
    final_states.push(FinalState::new(4, None));
    final_states.push(FinalState::new(3, Some(0.67)));

    let s0 = VectorFstState {
        final_weight: None,
        arcs: vec![Arc::new(32, 32, ProbabilityWeight::one(), 1)],
    };

    let s1 = VectorFstState {
        final_weight: None,
        arcs: vec![Arc::new(45, 45, ProbabilityWeight::one(), 2)],
    };

    let s2 = VectorFstState {
        final_weight: None,
        arcs: vec![Arc::new(18, 18, ProbabilityWeight::new(0.25), 3)],
    };

    let s3 = VectorFstState {
        final_weight: Some(ProbabilityWeight::new(0.67)),
        arcs: vec![Arc::new(45, 45, ProbabilityWeight::one(), 4)],
    };

    let s4 = VectorFstState {
        final_weight: Some(ProbabilityWeight::one()),
        arcs: vec![Arc::new(5, 5, ProbabilityWeight::new(0.31), 5)],
    };

    let s5 = VectorFstState {
        final_weight: Some(ProbabilityWeight::one()),
        arcs: vec![],
    };

    let vector_fst = VectorFst {
        start_state: Some(0),
        states: vec![s0, s1, s2, s3, s4, s5],
    };

    TextParserTest {
        name: "test_fst_002".to_string(),
        path: rel_to_abs_path!("text_fst_002.txt"),
        parsed_text_fst: ParsedTextFst {
            start_state: Some(0),
            transitions,
            final_states,
        },
        vector_fst,
    }
}
