use nom::types::CompleteStr;
use nom::{digit1, float};

use crate::parsers::text::parsed_text_fst::{FinalState, ParsedTextFst, Transition};

fn str_to_usize(input: CompleteStr) -> Result<usize, std::num::ParseIntError> {
    (*input).parse()
}

named!(num <CompleteStr, usize>, map_res!(digit1, str_to_usize));

named!(optional_weight <CompleteStr, Option<f32>>, opt!(preceded!(tag!("\t"), float)));

named!(transition <CompleteStr, Transition>, do_parse!(
    state: num >>
    tag!("\t") >>
    nextstate: num >>
    tag!("\t") >>
    ilabel: num >>
    tag!("\t") >>
    olabel: num >>
    weight: optional_weight >>
    (Transition {
        state, ilabel, olabel, weight, nextstate})
));

named!(vec_transitions <CompleteStr, Vec<Transition>>,
    many0!(terminated!(transition, tag!("\n")))
);

named!(final_state <CompleteStr, FinalState>, do_parse!(
    state: num >>
    weight: optional_weight >>
    (FinalState {state, weight})
));

named!(vec_final_states <CompleteStr, Vec<FinalState>>,
    many0!(terminated!(final_state, tag!("\n")))
);

named!(pub parse_text_fst <CompleteStr, ParsedTextFst>, do_parse!(
    transitions: vec_transitions >>
    final_states: vec_final_states >>
    (ParsedTextFst{transitions, final_states: final_states}))
);
