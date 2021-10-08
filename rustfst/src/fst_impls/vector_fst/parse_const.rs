use std::fs::read;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use nom::bytes::complete::take;
use nom::multi::count;
use nom::number::complete::le_i32;
use nom::IResult;

use crate::fst_impls::const_fst::{
    CONST_ALIGNED_FILE_VERSION, CONST_ARCH_ALIGNMENT, CONST_MIN_FILE_VERSION,
};
use crate::fst_impls::vector_fst::VectorFstState;
use crate::fst_impls::{ConstFst, VectorFst};
use crate::fst_properties::FstProperties;
use crate::fst_traits::SerializableFst;
use crate::parsers::bin_fst::fst_header::FstHeader;
use crate::parsers::bin_fst::utils_parsing::{
    parse_bin_fst_tr, parse_final_weight, parse_start_state,
};
use crate::parsers::nom_utils::NomCustomError;
use crate::semirings::SerializableSemiring;
use crate::{Tr, TrsVec};

impl<W: SerializableSemiring> VectorFst<W> {
    /// Load a VectorFst directly from a ConstFst file.
    pub fn read_from_const<P: AsRef<Path>>(path_bin_fst: P) -> Result<Self> {
        let data = read(path_bin_fst.as_ref()).with_context(|| {
            format!(
                "Can't open ConstFst binary file : {:?}",
                path_bin_fst.as_ref()
            )
        })?;

        let (_, parsed_fst) = parse_const_fst(&data)
            .map_err(|_| format_err!("Error while parsing binary ConstFst file as a VectorFst"))?;

        Ok(parsed_fst)
    }
}

struct TempState<W> {
    final_weight: Option<W>,
    ntrs: usize,
    niepsilons: usize,
    noepsilons: usize,
}

fn parse_const_state<W: SerializableSemiring>(
    i: &[u8],
) -> IResult<&[u8], TempState<W>, NomCustomError<&[u8]>> {
    let (i, final_weight) = W::parse_binary(i)?;
    let (i, _pos) = le_i32(i)?;
    let (i, ntrs) = le_i32(i)?;
    let (i, niepsilons) = le_i32(i)?;
    let (i, noepsilons) = le_i32(i)?;

    Ok((
        i,
        TempState {
            final_weight: parse_final_weight(final_weight),
            ntrs: ntrs as usize,
            niepsilons: niepsilons as usize,
            noepsilons: noepsilons as usize,
        },
    ))
}

fn parse_const_fst<W: SerializableSemiring>(
    i: &[u8],
) -> IResult<&[u8], VectorFst<W>, NomCustomError<&[u8]>> {
    let stream_len = i.len();

    let (mut i, hdr) = FstHeader::parse(
        i,
        CONST_MIN_FILE_VERSION,
        // Intentional as the ConstFst file is being parsed.
        ConstFst::<W>::fst_type(),
        Tr::<W>::tr_type(),
    )?;
    let aligned = hdr.version == CONST_ALIGNED_FILE_VERSION;
    let pos = stream_len - i.len();

    // Align input
    if aligned && hdr.num_states > 0 && pos % CONST_ARCH_ALIGNMENT > 0 {
        i = take(CONST_ARCH_ALIGNMENT - (pos % CONST_ARCH_ALIGNMENT))(i)?.0;
    }
    let (mut i, temp_states) = count(parse_const_state::<W>, hdr.num_states as usize)(i)?;
    let pos = stream_len - i.len();

    // Align input
    if aligned && hdr.num_trs > 0 && pos % CONST_ARCH_ALIGNMENT > 0 {
        i = take(CONST_ARCH_ALIGNMENT - (pos % CONST_ARCH_ALIGNMENT))(i)?.0;
    }

    let mut vector_states = Vec::with_capacity(temp_states.len());
    for temp_state in temp_states {
        let (j, trs) = count(parse_bin_fst_tr, temp_state.ntrs)(i)?;
        i = j;
        vector_states.push(VectorFstState {
            final_weight: temp_state.final_weight,
            trs: TrsVec(Arc::new(trs)),
            niepsilons: temp_state.niepsilons,
            noepsilons: temp_state.noepsilons,
        });
    }

    Ok((
        i,
        VectorFst {
            start_state: parse_start_state(hdr.start),
            states: vector_states,
            isymt: hdr.isymt,
            osymt: hdr.osymt,
            properties: FstProperties::from_bits_truncate(hdr.properties),
        },
    ))
}
