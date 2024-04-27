use std::io::Write;

use anyhow::Result;
use nom::bytes::complete::take;
use nom::combinator::{map_res, verify};
use nom::IResult;

use bitflags::bitflags;

use crate::parsers::bin_symt::nom_parser::{parse_symbol_table_bin, write_bin_symt};
use crate::parsers::nom_utils::NomCustomError;
use crate::parsers::{parse_bin_i32, parse_bin_i64, parse_bin_u32, parse_bin_u64};
use crate::parsers::{write_bin_i32, write_bin_i64, write_bin_u32, write_bin_u64};
use crate::SymbolTable;
use std::sync::Arc;

// Identifies stream data as an FST (and its endianity).
pub(crate) static FST_MAGIC_NUMBER: i32 = 2_125_659_606;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct FstFlags: u32 {
        const HAS_ISYMBOLS = 0b1;
        const HAS_OSYMBOLS = 0b1 << 1;
        const IS_ALIGNED = 0b1 << 2;
    }
}

#[derive(Debug)]
pub(crate) struct FstHeader {
    pub(crate) magic_number: i32,
    pub(crate) fst_type: OpenFstString,
    pub(crate) tr_type: OpenFstString,
    pub(crate) version: i32,
    pub(crate) flags: FstFlags,
    pub(crate) properties: u64,
    pub(crate) start: i64,
    pub(crate) num_states: i64,
    pub(crate) num_trs: i64,
    pub(crate) isymt: Option<Arc<SymbolTable>>,
    pub(crate) osymt: Option<Arc<SymbolTable>>,
}

#[derive(Debug)]
pub(crate) struct OpenFstString {
    n: i32,
    s: String,
}

fn optionally_parse_symt(
    i: &[u8],
    parse_symt: bool,
) -> IResult<&[u8], Option<SymbolTable>, NomCustomError<&[u8]>> {
    if parse_symt {
        let (i, symt) = parse_symbol_table_bin(i)?;
        Ok((i, Some(symt)))
    } else {
        Ok((i, None))
    }
}

fn optionally_write_symt<W: Write>(file: &mut W, symt: &Option<Arc<SymbolTable>>) -> Result<()> {
    if let Some(symt) = symt {
        write_bin_symt(file, symt)
    } else {
        Ok(())
    }
}

impl FstHeader {
    pub(crate) fn parse<S1: AsRef<str>, S2: AsRef<str>>(
        i: &[u8],
        min_file_version: i32,
        fst_loading_type: S1,
        tr_loading_type: S2,
    ) -> IResult<&[u8], FstHeader, NomCustomError<&[u8]>> {
        let (i, magic_number) = verify(parse_bin_i32, |v: &i32| *v == FST_MAGIC_NUMBER)(i)?;
        let (i, fst_type) = verify(OpenFstString::parse, |v| {
            v.s.as_str() == fst_loading_type.as_ref()
        })(i)?;
        let (i, tr_type) = verify(OpenFstString::parse, |v| {
            v.s.as_str() == tr_loading_type.as_ref()
        })(i)?;
        let (i, version) = verify(parse_bin_i32, |v: &i32| *v >= min_file_version)(i)?;
        let (i, flags) = map_res(parse_bin_u32, |v: u32| {
            FstFlags::from_bits(v).ok_or("Could not parse Fst Flags")
        })(i)?;
        let (i, properties) = parse_bin_u64(i)?;
        let (i, start) = parse_bin_i64(i)?;
        let (i, num_states) = parse_bin_i64(i)?;
        let (i, num_trs) = parse_bin_i64(i)?;

        let (i, isymt) = optionally_parse_symt(i, flags.contains(FstFlags::HAS_ISYMBOLS))?;
        let (i, osymt) = optionally_parse_symt(i, flags.contains(FstFlags::HAS_OSYMBOLS))?;

        Ok((
            i,
            FstHeader {
                magic_number,
                fst_type,
                tr_type,
                version,
                flags,
                properties,
                start,
                num_states,
                num_trs,
                isymt: isymt.map(Arc::new),
                osymt: osymt.map(Arc::new),
            },
        ))
    }

    pub(crate) fn write<W: Write>(&self, file: &mut W) -> Result<()> {
        //magic_number: i32,
        write_bin_i32(file, self.magic_number)?;
        //fst_type: OpenFstString,
        self.fst_type.write(file)?;
        //tr_type: OpenFstString,
        self.tr_type.write(file)?;
        //version: i32,
        write_bin_i32(file, self.version)?;
        //flags: i32,
        write_bin_u32(file, self.flags.bits())?;
        //properties: u64
        write_bin_u64(file, self.properties)?;
        //start: i64,
        write_bin_i64(file, self.start)?;
        //num_states: i64,
        write_bin_i64(file, self.num_states)?;
        //num_trs: i64,
        write_bin_i64(file, self.num_trs)?;
        optionally_write_symt(file, &self.isymt)?;
        optionally_write_symt(file, &self.osymt)?;
        Ok(())
    }
}

impl OpenFstString {
    pub(crate) fn new<I: Into<String>>(s: I) -> Self {
        let _s = s.into();
        OpenFstString {
            n: _s.len() as i32,
            s: _s,
        }
    }
    pub(crate) fn parse(i: &[u8]) -> IResult<&[u8], OpenFstString, NomCustomError<&[u8]>> {
        let (i, n) = parse_bin_i32(i)?;
        let (i, s) = take(n as usize)(i)?;
        Ok((
            i,
            OpenFstString {
                n,
                s: String::from_utf8(s.to_vec()).unwrap(),
            },
        ))
    }

    pub(crate) fn write<W: Write>(&self, file: &mut W) -> Result<()> {
        write_bin_i32(file, self.n)?;
        file.write_all(self.s.as_bytes()).map_err(|e| e.into())
    }
}

impl From<OpenFstString> for String {
    fn from(os: OpenFstString) -> Self {
        os.s
    }
}
