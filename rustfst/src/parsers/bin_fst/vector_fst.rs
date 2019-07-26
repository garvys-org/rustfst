use std::fs::read;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

use failure::{Fallible, ResultExt};
use nom::IResult;
use nom::{le_f32, le_i32, le_i64, le_u64};

use crate::fst_impls::vector::vector_fst::VectorFstState;
use crate::fst_impls::VectorFst;
use crate::fst_traits::{ArcIterator, BinaryDeserializer, BinarySerializer, CoreFst, ExpandedFst};
use crate::semirings::Semiring;
use crate::Arc;
use crate::StateId;

// Identifies stream data as an FST (and its endianity).
static FST_MAGIC_NUMBER: i32 = 2_125_659_606;
static MIN_FILE_VERSION: i32 = 2;
static NO_STATE_ID: i32 = -1;

#[derive(Debug)]
struct FstHeader {
    magic_number: i32,
    fst_type: OpenFstString,
    arc_type: OpenFstString,
    version: i32,
    flags: i32,
    properties: u64,
    start: i64,
    num_states: i64,
    num_arcs: i64,
}

#[derive(Debug)]
struct OpenFstString {
    n: i32,
    s: String,
}

#[derive(Debug, PartialEq)]
struct Transition {
    ilabel: i32,
    olabel: i32,
    weight: f32,
    nextstate: i32,
}

named!(parse_kaldi_string <&[u8], OpenFstString>, do_parse!(
    n: le_i32 >>
    s: take!(n as usize) >>
    (OpenFstString{n, s: String::from_utf8(s.to_vec()).unwrap()}))
);

named!(parse_fst_header <&[u8], FstHeader>, do_parse!(
    magic_number: verify!(le_i32, |v: i32| v == FST_MAGIC_NUMBER) >>
    fst_type: parse_kaldi_string >>
    arc_type: parse_kaldi_string >>
    version: verify!(le_i32, |v: i32| v >= MIN_FILE_VERSION) >>
    flags: le_i32 >>
    properties: le_u64 >>
    start: le_i64 >>
    num_states: le_i64 >>
    num_arcs: le_i64 >>
    (FstHeader {magic_number, fst_type, arc_type, version, flags, properties, start, num_states, num_arcs}))
);

fn parse_fst_arc<W: Semiring<Type = f32>>(i: &[u8]) -> IResult<&[u8], Arc<W>, u32> {
    do_parse!(
        i,
        ilabel: le_i32
            >> olabel: le_i32
            >> weight: le_f32
            >> nextstate: le_i32
            >> (Arc {
                ilabel: ilabel as usize,
                olabel: olabel as usize,
                weight: W::new(weight),
                nextstate: nextstate as usize
            })
    )
}

fn parse_fst_state<W: Semiring<Type = f32>>(i: &[u8]) -> IResult<&[u8], VectorFstState<W>, u32> {
    do_parse!(
        i,
        final_weight: le_f32
            >> num_arcs: le_i64
            >> arcs: count!(parse_fst_arc, num_arcs as usize)
            >> (VectorFstState {
                final_weight: parse_final_weight(final_weight),
                arcs
            })
    )
}

fn parse_fst<W: Semiring<Type = f32>>(i: &[u8]) -> IResult<&[u8], VectorFst<W>, u32> {
    do_parse!(
        i,
        header: parse_fst_header
            >> states: count!(parse_fst_state, header.num_states as usize)
            >> (VectorFst {
                start_state: parse_start_state(header.start),
                states: states
            })
    )
}

fn complete_parse_fst<W: Semiring<Type = f32>>(i: &[u8]) -> IResult<&[u8], VectorFst<W>, u32> {
    complete!(i, parse_fst)
}

#[inline]
fn parse_start_state(s: i64) -> Option<StateId> {
    if s == i64::from(NO_STATE_ID) {
        None
    } else {
        Some(s as StateId)
    }
}

#[inline]
fn parse_final_weight<W: Semiring<Type = f32>>(w: f32) -> Option<W> {
    // TODO: Avoid this re-allocation
    let zero_weight = W::zero().take_value();
    if w != zero_weight {
        Some(W::new(w))
    } else {
        None
    }
}

impl<W: Semiring<Type = f32> + 'static> BinaryDeserializer for VectorFst<W> {
    fn read<P: AsRef<Path>>(path_bin_fst: P) -> Fallible<Self> {
        let data = read(path_bin_fst.as_ref()).with_context(|_| {
            format!("Can't open FST binary file : {:?}", path_bin_fst.as_ref())
        })?;

        let (_, parsed_fst) = complete_parse_fst(&data)
            .map_err(|_| format_err!("Error while parsing binary VectorFst"))?;

        Ok(parsed_fst)
    }
}

#[inline]
fn write_bin_i32<W: Write>(file: &mut W, i: i32) -> Fallible<()> {
    file.write_all(&i.to_le_bytes()).map_err(|e| e.into())
}

#[inline]
fn write_bin_u64<W: Write>(file: &mut W, i: u64) -> Fallible<()> {
    file.write_all(&i.to_le_bytes()).map_err(|e| e.into())
}

#[inline]
fn write_bin_i64<W: Write>(file: &mut W, i: i64) -> Fallible<()> {
    file.write_all(&i.to_le_bytes()).map_err(|e| e.into())
}

#[inline]
fn write_bin_f32<W: Write>(file: &mut W, i: f32) -> Fallible<()> {
    file.write_all(&i.to_bits().to_le_bytes())
        .map_err(|e| e.into())
}

#[inline]
fn write_bin_string<W: Write>(file: &mut W, s: &str) -> Fallible<()> {
    write_bin_i32(file, s.len() as i32)?;
    file.write_all(s.as_bytes()).map_err(|e| e.into())
}

impl<W: 'static + Semiring<Type = f32>> BinarySerializer for VectorFst<W> {
    fn write<P: AsRef<Path>>(&self, path_bin_fst: P) -> Fallible<()> {
        let mut file = BufWriter::new(File::create(path_bin_fst)?);

        // FstHeader
        //magic_number: i32,
        write_bin_i32(&mut file, FST_MAGIC_NUMBER)?;
        //fst_type: OpenFstString,
        write_bin_string(&mut file, "vector")?;
        //arc_type: OpenFstString,
        // FIXME: This should be generated by the weight type
        write_bin_string(&mut file, "standard")?;
        //version: i32,
        write_bin_i32(&mut file, 2i32)?;
        //flags: i32,
        // FIXME: Flags are used to check whether or not a symboltable has to be loaded
        write_bin_i32(&mut file, 0i32)?;
        //properties: u64, 3 = kMutable | kExpanded
        // FIXME: Once the properties are stored, need to read them
        write_bin_u64(&mut file, 3u64)?;
        //start: i64,
        write_bin_i64(&mut file, self.start_state.map(|v| v as i64).unwrap_or(-1))?;
        //num_states: i64,
        write_bin_i64(&mut file, self.num_states() as i64)?;
        //num_arcs: i64,
        let num_arcs: usize = (0..self.num_states())
            .map(|s: usize| unsafe { self.num_arcs_unchecked(s) })
            .sum();
        write_bin_i64(&mut file, num_arcs as i64)?;

        let zero = W::zero();
        // FstBody
        for state in 0..self.num_states() {
            let f_weight = self.final_weight(state).unwrap_or_else(|| &zero).value();
            write_bin_f32(&mut file, *f_weight)?;
            write_bin_i64(&mut file, unsafe { self.num_arcs_unchecked(state) } as i64)?;

            for arc in unsafe { self.arcs_iter_unchecked(state) } {
                write_bin_i32(&mut file, arc.ilabel as i32)?;
                write_bin_i32(&mut file, arc.olabel as i32)?;
                let weight = arc.weight.value();
                write_bin_f32(&mut file, *weight)?;
                write_bin_i32(&mut file, arc.nextstate as i32)?;
            }
        }

        Ok(())
    }
}
