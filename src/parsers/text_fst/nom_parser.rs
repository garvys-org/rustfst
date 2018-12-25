use nom::float;
use nom::types::CompleteStr;

use crate::parsers::nom_utils::num;
use crate::parsers::text_fst::parsed_text_fst::{FinalState, ParsedTextFst, RowParsed, Transition};

named!(optional_weight <CompleteStr, Option<f32>>, opt!(preceded!(tag!("\t"), float)));

named!(transition <CompleteStr, RowParsed>, do_parse!(
    state: num >>
    tag!("\t") >>
    nextstate: num >>
    tag!("\t") >>
    ilabel: num >>
    tag!("\t") >>
    olabel: num >>
    weight: optional_weight >>
    (RowParsed::Transition(Transition {
        state, ilabel, olabel, weight, nextstate}))
));

named!(final_state <CompleteStr, RowParsed>, do_parse!(
    state: num >>
    weight: optional_weight >>
    (RowParsed::FinalState(FinalState {state, weight}))
));

named!(row_parsed <CompleteStr, RowParsed>, alt!(transition | final_state));

named!(pub vec_rows_parsed <CompleteStr, Vec<RowParsed>>,
 separated_list!(tag!("\n"), row_parsed)
);
