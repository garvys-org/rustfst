use std::fs::read;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

use failure::{Fallible, ResultExt};
use nom::bytes::complete::take;
use nom::combinator::verify;
use nom::multi::count;
use nom::number::complete::{le_f32, le_i32, le_i64, le_u64};
use nom::IResult;

use crate::fst_impls::vector_fst::VectorFstState;
use crate::fst_impls::VectorFst;
use crate::fst_traits::{ArcIterator, BinaryDeserializer, BinarySerializer, CoreFst, ExpandedFst};
use crate::semirings::Semiring;
use crate::StateId;
use crate::{Arc, NO_STATE_ID};

// Identifies stream data as an FST (and its endianity).
pub(crate) static FST_MAGIC_NUMBER: i32 = 2_125_659_606;
// TODO: Make this min_file_version fst_type dependent
static MIN_FILE_VERSION: i32 = 2;

// TODO: Perform also the writing here
#[derive(Debug)]
pub(crate) struct FstHeader {
    pub(crate) magic_number: i32,
    pub(crate) fst_type: OpenFstString,
    pub(crate) arc_type: OpenFstString,
    pub(crate) version: i32,
    pub(crate) flags: i32,
    pub(crate) properties: u64,
    pub(crate) start: i64,
    pub(crate) num_states: i64,
    pub(crate) num_arcs: i64,
}

#[derive(Debug)]
pub(crate) struct OpenFstString {
    n: i32,
    s: String,
}

pub(crate) fn parse_kaldi_string(i: &[u8]) -> IResult<&[u8], OpenFstString> {
    let (i, n) = le_i32(i)?;
    let (i, s) = take(n as usize)(i)?;
    Ok((
        i,
        OpenFstString {
            n,
            s: String::from_utf8(s.to_vec()).unwrap(),
        },
    ))
}

pub(crate) fn parse_fst_header(i: &[u8]) -> IResult<&[u8], FstHeader> {
    let (i, magic_number) = verify(le_i32, |v: &i32| *v == FST_MAGIC_NUMBER)(i)?;
    let (i, fst_type) = parse_kaldi_string(i)?;
    let (i, arc_type) = parse_kaldi_string(i)?;
    let (i, version) = verify(le_i32, |v: &i32| *v >= MIN_FILE_VERSION)(i)?;
    let (i, flags) = le_i32(i)?;
    let (i, properties) = le_u64(i)?;
    let (i, start) = le_i64(i)?;
    let (i, num_states) = le_i64(i)?;
    let (i, num_arcs) = le_i64(i)?;
    Ok((
        i,
        FstHeader {
            magic_number,
            fst_type,
            arc_type,
            version,
            flags,
            properties,
            start,
            num_states,
            num_arcs,
        },
    ))
}
