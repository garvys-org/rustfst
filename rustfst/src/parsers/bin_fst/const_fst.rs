use std::fs::read;
use std::path::Path;

use failure::{Fallible, ResultExt};
use nom::bytes::complete::take;
use nom::combinator::verify;
use nom::multi::count;
use nom::number::complete::{le_f32, le_i32, le_i64, le_u64};
use nom::IResult;

use crate::fst_impls::const_fst::ConstState;
use crate::fst_impls::ConstFst;
use crate::fst_traits::BinaryDeserializer;
use crate::parsers::bin_fst::fst_header::parse_fst_header;
use crate::parsers::bin_fst::utils::{parse_final_weight, parse_fst_arc, parse_start_state};
use crate::semirings::Semiring;

static MIN_FILE_VERSION: i32 = 2;

fn parse_const_state<W: Semiring<Type = f32>>(i: &[u8]) -> IResult<&[u8], ConstState<W>> {
    let (i, final_weight) = le_f32(i)?;
    let (i, pos) = le_i32(i)?;
    let (i, narcs) = le_i32(i)?;

    // TODO: Use niepsilons and noepsilons
    let (i, _niepsilons) = le_i32(i)?;
    let (i, _noepsilons) = le_i32(i)?;

    Ok((
        i,
        ConstState {
            final_weight: parse_final_weight(final_weight),
            pos: pos as usize,
            narcs: narcs as usize,
        },
    ))
}

fn parse_const_fst<W: Semiring<Type = f32>>(i: &[u8]) -> IResult<&[u8], ConstFst<W>> {
    let (i, hdr) = parse_fst_header(i)?;
    let (i, const_states) = count(parse_const_state, hdr.num_states as usize)(i)?;
    let (i, const_arcs) = count(parse_fst_arc, hdr.num_arcs as usize)(i)?;

    Ok((
        i,
        ConstFst {
            start: parse_start_state(hdr.start),
            states: const_states,
            arcs: const_arcs,
        },
    ))
}

impl<W: Semiring<Type = f32> + 'static> BinaryDeserializer for ConstFst<W> {
    fn read<P: AsRef<Path>>(path_bin_fst: P) -> Fallible<Self> {
        let data = read(path_bin_fst.as_ref()).with_context(|_| {
            format!(
                "Can't open ConstFst binary file : {:?}",
                path_bin_fst.as_ref()
            )
        })?;

        let (data, parsed_fst) = parse_const_fst(&data)
            .map_err(|_| format_err!("Error while parsing binary ConstFst"))?;

        Ok(parsed_fst)
    }
}
