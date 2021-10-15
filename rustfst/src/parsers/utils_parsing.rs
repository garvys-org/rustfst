use nom::number::complete::{le_f32, le_i32, le_i64, le_u32, le_u64, le_u8};
use nom::IResult;

use crate::parsers::nom_utils::NomCustomError;

#[inline]
pub fn parse_bin_i64(i: &[u8]) -> IResult<&[u8], i64, NomCustomError<&[u8]>> {
    le_i64(i)
}

#[inline]
pub fn parse_bin_u64(i: &[u8]) -> IResult<&[u8], u64, NomCustomError<&[u8]>> {
    le_u64(i)
}

#[inline]
pub fn parse_bin_i32(i: &[u8]) -> IResult<&[u8], i32, NomCustomError<&[u8]>> {
    le_i32(i)
}

#[inline]
pub fn parse_bin_u32(i: &[u8]) -> IResult<&[u8], u32, NomCustomError<&[u8]>> {
    le_u32(i)
}

#[inline]
pub fn parse_bin_f32(i: &[u8]) -> IResult<&[u8], f32, NomCustomError<&[u8]>> {
    le_f32(i)
}

#[inline]
pub fn parse_bin_u8(i: &[u8]) -> IResult<&[u8], u8, NomCustomError<&[u8]>> {
    le_u8(i)
}
