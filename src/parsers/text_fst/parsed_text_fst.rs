use std::fs::read_to_string;
use std::path::Path;

use nom::types::CompleteStr;

use crate::parsers::text_fst::nom_parser::vec_rows_parsed;
use crate::Result as ResultRustfst;
use crate::{Label, StateId};

#[derive(Debug, PartialEq)]
pub enum RowParsed {
    Transition(Transition),
    FinalState(FinalState),
}

/// Struct representing a parsed fst in text format. It contains a vector of transitions
/// and a vector final states. The first state in the vector of transition is the start state.
/// This container doesn't depend on any Semiring.
#[derive(Debug, PartialEq, Default)]
pub struct ParsedTextFst {
    pub transitions: Vec<Transition>,
    pub final_states: Vec<FinalState>,
}

/// A transition is a five-tuple. There is one for each arc in the graph.
/// It contains, the `state` from which the arc is leaving and a `nextstate` which is the target.
/// Also there are both labels and weight stored on the arc.
/// Transitions without weight have a one weight in the Semiring.
#[derive(Debug, PartialEq)]
pub struct Transition {
    /// state from which the arc is leaving.
    pub state: StateId,
    /// Input label of the arc.
    pub ilabel: Label,
    /// Output label of the arc.
    pub olabel: Label,
    /// Weight on the arc.
    pub weight: Option<f32>,
    /// state reached by the arc.
    pub nextstate: StateId,
}

/// A final state is composed of a state and a final weight.
/// If the weight is missing there it has a one weight in the semiring.
#[derive(Debug, PartialEq)]
pub struct FinalState {
    pub state: StateId,
    pub weight: Option<f32>,
}

/// Removes the enum
impl Into<ParsedTextFst> for Vec<RowParsed> {
    fn into(self) -> ParsedTextFst {
        let mut parsed_fst = ParsedTextFst::default();

        for row_parsed in self.into_iter() {
            match row_parsed {
                RowParsed::Transition(t) => parsed_fst.transitions.push(t),
                RowParsed::FinalState(f) => parsed_fst.final_states.push(f),
            };
        }

        parsed_fst
    }
}

impl ParsedTextFst {
    /// Loads an FST from a loaded string in text format usually called `At&T FSM format`.
    ///
    /// # Format:
    ///
    /// ## Specification:
    ///
    /// Arc format: `src dest ilabel olabel [weight]`
    ///
    /// Final state format: `state [weight]`
    ///
    /// Lines may occur in any order except initial state must be first line.
    /// Unspecified weights default to 1.0 (for the Semiring).
    /// All the values are separated by a tabulation (`\t`).
    ///
    /// ## Example:
    /// ```text
    /// 0	1	32	32
    /// 1	2	45	45
    /// 2	3	18	18	0.25
    /// 3	4	45	45
    /// 4	5	5	5	0.31
    /// 3	0.67
    /// ```
    pub fn from_string(fst_string: &str) -> ResultRustfst<Self> {
        let complete_fst_str = CompleteStr(fst_string);
        let (_, vec_rows_parsed) = vec_rows_parsed(complete_fst_str)
            .map_err(|_| format_err!("Error while parsing text fst"))?;

        Ok(vec_rows_parsed.into())
    }

    /// Loads an FST from a serialized file in text format usually called `At&T FSM format`.
    ///
    /// # Format:
    ///
    /// ## Specification:
    /// Arc format: `src dest ilabel olabel [weight]`
    ///
    /// Final state format: `state [weight]`
    ///
    /// Lines may occur in any order except initial state must be first line.
    /// Unspecified weights default to 1.0 (for the Semiring).
    /// All the values are separated by a tabulation (`\t`).
    ///
    /// ## Example:
    /// ```text
    /// 0	1	32	32
    /// 1	2	45	45
    /// 2	3	18	18	0.25
    /// 3	4	45	45
    /// 4	5	5	5	0.31
    /// 3	0.67
    /// ```
    pub fn from_path<P: AsRef<Path>>(path_fst_text: P) -> ResultRustfst<Self> {
        let fst_string = read_to_string(path_fst_text)?;
        Self::from_string(&fst_string)
    }

    pub fn start(&self) -> Option<StateId> {
        self.transitions.first().map(|t| t.state)
    }

    pub fn num_states(&self) -> usize {
        let it_states = self.transitions.iter().map(|t| t.state);
        let it_nextstates = self.transitions.iter().map(|t| t.nextstate);
        let max_state = it_states.chain(it_nextstates).max();
        max_state.map(|n| n + 1).unwrap_or(0)
    }
}

impl Transition {
    pub fn new(
        state: StateId,
        ilabel: Label,
        olabel: Label,
        weight: Option<f32>,
        nextstate: StateId,
    ) -> Self {
        Self {
            state,
            ilabel,
            olabel,
            weight,
            nextstate,
        }
    }
}

impl FinalState {
    pub fn new(state: StateId, weight: Option<f32>) -> Self {
        Self { state, weight }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_data::text_fst::get_test_data_for_text_parser;

    #[test]
    fn test_parse_text_fst() -> ResultRustfst<()> {
        for data in get_test_data_for_text_parser() {
            let parsed_fst = ParsedTextFst::from_path(data.path)?;
            let parsed_fst_ref = data.parsed_text_fst;
            assert_eq!(
                parsed_fst, parsed_fst_ref,
                "Tests failing for parse text fst for wFST : {}",
                data.name
            );
        }
        Ok(())
    }

    #[test]
    fn test_parse_text_fst_not_contiguous() -> ResultRustfst<()> {
        // Check that parsingg transitions, then final states then transition is working
        let parsed_fst = ParsedTextFst::from_string("0\t2\t0\t0\n1\n2\t1\t12\t25\n")?;

        let mut transitions = vec![];
        transitions.push(Transition::new(0, 0, 0, None, 2));
        transitions.push(Transition::new(2, 12, 25, None, 1));

        let mut final_states = vec![];
        final_states.push(FinalState::new(1, None));

        let parsed_fst_ref = ParsedTextFst {
            transitions,
            final_states,
        };

        assert_eq!(parsed_fst, parsed_fst_ref);

        Ok(())
    }

}
