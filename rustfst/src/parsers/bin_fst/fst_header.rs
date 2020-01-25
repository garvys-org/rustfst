use std::io::Write;

use failure::Fallible;
use nom::bytes::complete::take;
use nom::combinator::{map_res, verify};
use nom::number::complete::{le_i32, le_i64, le_u32, le_u64};
use nom::IResult;

use bitflags::bitflags;

use crate::parsers::bin_fst::utils_serialization::{
    write_bin_i32, write_bin_i64, write_bin_u32, write_bin_u64,
};
use crate::parsers::bin_symt::nom_parser::parse_symbol_table_bin;
use crate::SymbolTable;
use std::rc::Rc;

// Identifies stream data as an FST (and its endianity).
pub(crate) static FST_MAGIC_NUMBER: i32 = 2_125_659_606;

bitflags! {
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
    pub(crate) arc_type: OpenFstString,
    pub(crate) version: i32,
    pub(crate) flags: FstFlags,
    pub(crate) properties: u64,
    pub(crate) start: i64,
    pub(crate) num_states: i64,
    pub(crate) num_arcs: i64,
    pub(crate) isymt: Option<Rc<SymbolTable>>,
    pub(crate) osymt: Option<Rc<SymbolTable>>,
}

#[derive(Debug)]
pub(crate) struct OpenFstString {
    n: i32,
    s: String,
}

fn optionally_parse_symt(i: &[u8], parse_symt: bool) -> IResult<&[u8], Option<SymbolTable>> {
    if parse_symt {
        let (i, symt) = parse_symbol_table_bin(i)?;
        Ok((i, Some(symt)))
    } else {
        Ok((i, None))
    }
}

impl FstHeader {
    pub(crate) fn parse<S1: AsRef<str>, S2: AsRef<str>>(
        i: &[u8],
        min_file_version: i32,
        fst_loading_type: S1,
        arc_loading_type: S2,
    ) -> IResult<&[u8], FstHeader> {
        let (i, magic_number) = verify(le_i32, |v: &i32| *v == FST_MAGIC_NUMBER)(i)?;
        let (i, fst_type) = verify(OpenFstString::parse, |v| {
            v.s.as_str() == fst_loading_type.as_ref()
        })(i)?;
        let (i, arc_type) = verify(OpenFstString::parse, |v| {
            v.s.as_str() == arc_loading_type.as_ref()
        })(i)?;
        let (i, version) = verify(le_i32, |v: &i32| *v >= min_file_version)(i)?;
        let (i, flags) = map_res(le_u32, |v: u32| {
            FstFlags::from_bits(v).ok_or_else(|| "Could not parse Fst Flags")
        })(i)?;
        let (i, properties) = le_u64(i)?;
        let (i, start) = le_i64(i)?;
        let (i, num_states) = le_i64(i)?;
        let (i, num_arcs) = le_i64(i)?;

        let (i, isymt) = optionally_parse_symt(i, flags.contains(FstFlags::HAS_ISYMBOLS))?;
        let (i, osymt) = optionally_parse_symt(i, flags.contains(FstFlags::HAS_OSYMBOLS))?;

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
                isymt: isymt.map(Rc::new),
                osymt: osymt.map(Rc::new),
            },
        ))
    }

    pub(crate) fn write<W: Write>(&self, file: &mut W) -> Fallible<()> {
        //magic_number: i32,
        write_bin_i32(file, self.magic_number)?;
        //fst_type: OpenFstString,
        self.fst_type.write(file)?;
        //arc_type: OpenFstString,
        self.arc_type.write(file)?;
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
        //num_arcs: i64,
        write_bin_i64(file, self.num_arcs)?;
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
    pub(crate) fn parse(i: &[u8]) -> IResult<&[u8], OpenFstString> {
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

    pub(crate) fn write<W: Write>(&self, file: &mut W) -> Fallible<()> {
        write_bin_i32(file, self.n)?;
        file.write_all(self.s.as_bytes()).map_err(|e| e.into())
    }
}

impl Into<String> for OpenFstString {
    fn into(self) -> String {
        self.s
    }
}
