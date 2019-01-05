use nom::digit1;
use nom::types::CompleteStr;

fn str_to_usize(input: CompleteStr) -> Result<usize, std::num::ParseIntError> {
    (*input).parse()
}

named!(pub num <CompleteStr, usize>, map_res!(digit1, str_to_usize));

named!(pub word <CompleteStr, String>, do_parse!(
    letters: take_while!(|c:char| (c != ' ') && (c != '\t') && (c != '\n')) >> (letters.to_string())));
