use std::fs::read_to_string;
use std::path::Path;

use anyhow::Result;

use crate::parsers::text_fst::nom_parser::vec_rows_parsed;
use crate::semirings::SerializableSemiring;
use crate::{Label, StateId};

#[derive(Debug, PartialEq)]
pub enum RowParsed<W: SerializableSemiring> {
    Transition(Transition<W>),
    FinalState(FinalState<W>),
    InfinityFinalState(StateId),
}

/// Struct representing a parsed fst in text format. It contains a vector of transitions
/// and a vector final states. The first state in the vector of transition is the start state.
/// This container doesn't depend on any Semiring.
#[derive(Debug, PartialEq)]
pub struct ParsedTextFst<W: SerializableSemiring> {
    pub transitions: Vec<Transition<W>>,
    pub final_states: Vec<FinalState<W>>,
    pub start_state: Option<StateId>,
}

/// A transition is a five-tuple. There is one for each transition in the graph.
/// It contains the `state` from which the transition is leaving and a `nextstate` which is the target.
/// Also there are both labels and weight stored on the transition.
/// Transitions without weight have a one weight in the Semiring.
#[derive(Debug, PartialEq)]
pub struct Transition<W: SerializableSemiring> {
    /// state from which the transition is leaving.
    pub state: StateId,
    /// Input label of the transition.
    pub ilabel: Label,
    /// Output label of the transition.
    pub olabel: Label,
    /// Weight on the transition.
    pub weight: Option<W>,
    /// state reached by the transition.
    pub nextstate: StateId,
}

/// A final state is composed of a state and a final weight.
/// If the weight is missing there it has a one weight in the semiring.
#[derive(Debug, PartialEq)]
pub struct FinalState<W: SerializableSemiring> {
    pub state: StateId,
    pub weight: Option<W>,
}

impl<W: SerializableSemiring> Default for ParsedTextFst<W> {
    fn default() -> Self {
        Self {
            transitions: vec![],
            final_states: vec![],
            start_state: None,
        }
    }
}

impl<W: SerializableSemiring> ParsedTextFst<W> {
    /// Loads an FST from a loaded string in text format usually called `At&T FSM format`.
    ///
    /// # Format:
    ///
    /// ## Specification:
    ///
    /// Tr format: `src dest ilabel olabel [weight]`
    ///
    /// Final state format: `state [weight]`
    ///
    /// Lines may occur in any order except initial state must be first line.
    /// Unspecified weights default to 1.0 (for the Semiring).
    /// All the values are separated by a tabulation (`\t`).
    ///
    /// ## Example:
    /// ```text
    /// 0   1   32  32
    /// 1   2   45  45
    /// 2   3   18  18  0.25
    /// 3   4   45  45
    /// 4   5   5   5   0.31
    /// 3   0.67
    /// ```
    pub fn from_string(fst_string: &str) -> Result<Self> {
        let (_, vec_rows_parsed) =
            vec_rows_parsed(fst_string).map_err(|_| format_err!("Error while parsing text fst"))?;

        Ok(Self::from_vec_rows_parsed(vec_rows_parsed))
    }

    fn from_vec_rows_parsed(v: Vec<RowParsed<W>>) -> Self {
        let mut parsed_fst = Self {
            start_state: v.first().map(|v| match v {
                RowParsed::Transition(t) => t.state,
                RowParsed::FinalState(f) => f.state,
                RowParsed::InfinityFinalState(g) => *g,
            }),
            ..Default::default()
        };

        for row_parsed in v.into_iter() {
            match row_parsed {
                RowParsed::Transition(t) => parsed_fst.transitions.push(t),
                RowParsed::FinalState(f) => parsed_fst.final_states.push(f),
                RowParsed::InfinityFinalState(_) => {}
            };
        }

        parsed_fst
    }

    /// Loads an FST from a serialized file in text format usually called `At&T FSM format`.
    ///
    /// # Format:
    ///
    /// ## Specification:
    /// Tr format: `src dest ilabel olabel [weight]`
    ///
    /// Final state format: `state [weight]`
    ///
    /// Lines may occur in any order except initial state must be first line.
    /// Unspecified weights default to 1.0 (for the Semiring).
    /// All the values are separated by a tabulation (`\t`).
    ///
    /// ## Example:
    /// ```text
    /// 0   1   32  32
    /// 1   2   45  45
    /// 2   3   18  18  0.25
    /// 3   4   45  45
    /// 4   5   5   5   0.31
    /// 3   0.67
    /// ```
    pub fn from_path<P: AsRef<Path>>(path_fst_text: P) -> Result<Self> {
        let fst_string = read_to_string(path_fst_text)?;
        Self::from_string(&fst_string)
    }

    pub fn start(&self) -> Option<StateId> {
        self.start_state
    }

    pub fn num_states(&self) -> usize {
        let it_states = self.transitions.iter().map(|t| t.state);
        let it_nextstates = self.transitions.iter().map(|t| t.nextstate);
        let it_final_states = self.final_states.iter().map(|f| f.state);
        let it_start_state = self.start_state.iter().cloned();
        let max_state = it_states
            .chain(it_nextstates)
            .chain(it_final_states)
            .chain(it_start_state)
            .max();
        max_state.map(|n| n as usize + 1).unwrap_or(0)
    }
}

impl<W: SerializableSemiring> Transition<W> {
    pub fn new(
        state: StateId,
        ilabel: Label,
        olabel: Label,
        weight: Option<W>,
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

impl<W: SerializableSemiring> FinalState<W> {
    pub fn new(state: StateId, weight: Option<W>) -> Self {
        Self { state, weight }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semirings::{Semiring, TropicalWeight};

    #[test]
    fn test_parse_text_fst_not_contiguous() -> Result<()> {
        // Check that parsing transitions, then final states then transition is working
        let parsed_fst =
            ParsedTextFst::<TropicalWeight>::from_string("0\t2\t0\t0\n1\n2\t1\t12\t25\n")?;

        let mut transitions = vec![];
        transitions.push(Transition::new(0, 0, 0, None, 2));
        transitions.push(Transition::new(2, 12, 25, None, 1));

        let mut final_states = vec![];
        final_states.push(FinalState::new(1, None));

        let parsed_fst_ref = ParsedTextFst {
            start_state: Some(0),
            transitions,
            final_states,
        };

        assert_eq!(parsed_fst, parsed_fst_ref);

        Ok(())
    }

    #[test]
    fn test_parse_text_fst_not_finishing_with_eol() -> Result<()> {
        // Check that parsing transitions, then final states then transition is working
        let parsed_fst = ParsedTextFst::<TropicalWeight>::from_string("0\t1\t0\t0\n1")?;

        let mut transitions = vec![];
        transitions.push(Transition::new(0, 0, 0, None, 1));

        let mut final_states = vec![];
        final_states.push(FinalState::new(1, None));

        let parsed_fst_ref = ParsedTextFst {
            start_state: Some(0),
            transitions,
            final_states,
        };

        assert_eq!(parsed_fst, parsed_fst_ref);

        Ok(())
    }

    #[test]
    fn test_parse_text_fst_infinity_final_states() -> Result<()> {
        let parsed_fst =
            ParsedTextFst::<TropicalWeight>::from_string("0\t1\t12\t25\t0.3\n1\tInfinity\n0\t0\n")?;

        let mut transitions = vec![];
        transitions.push(Transition::new(
            0,
            12,
            25,
            Some(TropicalWeight::new(0.3)),
            1,
        ));

        let mut final_states = vec![];
        final_states.push(FinalState::new(0, Some(TropicalWeight::new(0.0))));

        let parsed_fst_ref = ParsedTextFst {
            start_state: Some(0),
            transitions,
            final_states,
        };

        assert_eq!(parsed_fst, parsed_fst_ref);

        Ok(())
    }
}
