use nom::bytes::complete::take_while;
use nom::character::complete::digit1;
use nom::combinator::map_res;
use nom::IResult;

pub fn num(i: &str) -> IResult<&str, usize> {
    map_res(digit1, |s: &str| s.parse())(i)
}

pub fn word(i: &str) -> IResult<&str, String> {
    let (i, letters) = take_while(|c: char| (c != ' ') && (c != '\t') && (c != '\n'))(i)?;
    Ok((i, letters.to_string()))
}
