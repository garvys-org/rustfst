use nom::types::CompleteStr;

use crate::parsers::nom_utils::{num, word};
use crate::parsers::text_symt::parsed_text_symt::ParsedTextSymt;
use crate::{Label, Symbol};

named!(row <CompleteStr, (Symbol, Label)>, do_parse!(
    symbol: word >>
    tag!("\t") >>
    label: num >>
    (symbol, label)
));

named!(vec_rows <CompleteStr, Vec<(Symbol, Label)>>,
    many0!(terminated!(row, tag!("\n")))
);

named!(pub(crate) parse_text_symt <CompleteStr, ParsedTextSymt>, do_parse!(
    pairs: vec_rows >>
    (ParsedTextSymt {pairs}))
);
