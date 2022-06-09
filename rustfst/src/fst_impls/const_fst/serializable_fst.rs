use std::io::Write;
use std::sync::Arc;

use anyhow::Result;
use itertools::Itertools;
use nom::bytes::complete::take;
use nom::multi::count;
use nom::IResult;

use crate::fst_impls::const_fst::data_structure::ConstState;
use crate::fst_impls::const_fst::{
    CONST_ALIGNED_FILE_VERSION, CONST_ARCH_ALIGNMENT, CONST_FILE_VERSION, CONST_MIN_FILE_VERSION,
};
use crate::fst_impls::ConstFst;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{ExpandedFst, Fst, SerializableFst};
use crate::parsers::bin_fst::fst_header::{FstFlags, FstHeader, OpenFstString, FST_MAGIC_NUMBER};
use crate::parsers::bin_fst::utils_parsing::{
    parse_bin_fst_tr, parse_final_weight, parse_start_state,
};
use crate::parsers::nom_utils::NomCustomError;
use crate::parsers::parse_bin_i32;
use crate::parsers::text_fst::ParsedTextFst;
use crate::parsers::write_bin_i32;
use crate::semirings::SerializableSemiring;
use crate::{Tr, EPS_LABEL};

impl<W: SerializableSemiring> SerializableFst<W> for ConstFst<W> {
    fn fst_type() -> String {
        "const".to_string()
    }

    fn load(data: &[u8]) -> Result<Self> {
        let (_, parsed_fst) = parse_const_fst(data)
            .map_err(|_| format_err!("Error while parsing binary ConstFst"))?;

        Ok(parsed_fst)
    }

    fn store<O: Write>(&self, mut output: O) -> Result<()> {
        let mut flags = FstFlags::empty();
        if self.input_symbols().is_some() {
            flags |= FstFlags::HAS_ISYMBOLS;
        }
        if self.output_symbols().is_some() {
            flags |= FstFlags::HAS_OSYMBOLS;
        }

        let hdr = FstHeader {
            magic_number: FST_MAGIC_NUMBER,
            fst_type: OpenFstString::new(Self::fst_type()),
            tr_type: OpenFstString::new(Tr::<W>::tr_type()),
            version: CONST_FILE_VERSION,
            // TODO: Set flags if the content is aligned
            flags,
            properties: self.properties.bits() | ConstFst::<W>::static_properties(),
            start: self.start.map(|v| v as i64).unwrap_or(-1),
            num_states: self.num_states() as i64,
            num_trs: self.trs.len() as i64,
            isymt: self.input_symbols().cloned(),
            osymt: self.output_symbols().cloned(),
        };
        hdr.write(&mut output)?;

        let zero = W::zero();
        for const_state in &self.states {
            let f_weight = const_state.final_weight.as_ref().unwrap_or(&zero);
            f_weight.write_binary(&mut output)?;

            write_bin_i32(&mut output, const_state.pos as i32)?;
            write_bin_i32(&mut output, const_state.ntrs as i32)?;
            write_bin_i32(&mut output, const_state.niepsilons as i32)?;
            write_bin_i32(&mut output, const_state.noepsilons as i32)?;
        }

        for tr in &*self.trs {
            write_bin_i32(&mut output, tr.ilabel as i32)?;
            write_bin_i32(&mut output, tr.olabel as i32)?;
            tr.weight.write_binary(&mut output)?;
            write_bin_i32(&mut output, tr.nextstate as i32)?;
        }

        Ok(())
    }

    fn from_parsed_fst_text(mut parsed_fst_text: ParsedTextFst<W>) -> Result<Self> {
        let start_state = parsed_fst_text.start();
        let num_states = parsed_fst_text.num_states();
        let num_trs = parsed_fst_text.transitions.len();

        let mut const_states = Vec::with_capacity(num_states);
        let mut const_trs = Vec::with_capacity(num_trs);

        parsed_fst_text.transitions.sort_by_key(|v| v.state);
        for (_state, tr_iterator) in parsed_fst_text
            .transitions
            .into_iter()
            .group_by(|v| v.state)
            .into_iter()
        {
            let pos = const_trs.len();
            // Some states might not have outgoing trs.
            const_states.resize_with(_state as usize, || ConstState {
                final_weight: None,
                pos,
                ntrs: 0,
                niepsilons: 0,
                noepsilons: 0,
            });
            let mut niepsilons = 0;
            let mut noepsilons = 0;
            const_trs.extend(tr_iterator.map(|v| {
                debug_assert_eq!(_state, v.state);
                let tr = Tr {
                    ilabel: v.ilabel,
                    olabel: v.olabel,
                    weight: v.weight.unwrap_or_else(W::one),
                    nextstate: v.nextstate,
                };
                if tr.ilabel == EPS_LABEL {
                    niepsilons += 1;
                }
                if tr.olabel == EPS_LABEL {
                    noepsilons += 1;
                }
                tr
            }));
            let num_trs_this_state = const_trs.len() - pos;
            const_states.push(ConstState::<W> {
                final_weight: None,
                pos,
                ntrs: num_trs_this_state,
                niepsilons,
                noepsilons,
            })
        }
        const_states.resize_with(num_states, || ConstState {
            final_weight: None,
            pos: const_trs.len(),
            ntrs: 0,
            niepsilons: 0,
            noepsilons: 0,
        });
        debug_assert_eq!(num_states, const_states.len());
        for final_state in parsed_fst_text.final_states.into_iter() {
            let weight = final_state.weight.unwrap_or_else(W::one);
            unsafe {
                const_states
                    .get_unchecked_mut(final_state.state as usize)
                    .final_weight = Some(weight)
            };
        }

        // Trick to compute the FstProperties. Indeed we need a fst to compute the properties
        // and we need the properties to construct a fst...
        let mut fst = ConstFst {
            states: const_states,
            trs: Arc::new(const_trs),
            start: start_state,
            isymt: None,
            osymt: None,
            properties: FstProperties::empty(),
        };

        let mut known = FstProperties::empty();
        let properties = crate::fst_properties::compute_fst_properties(
            &fst,
            FstProperties::all_properties(),
            &mut known,
            false,
        )?;
        fst.properties = properties;

        Ok(fst)
    }
}

fn parse_const_state<W: SerializableSemiring>(
    i: &[u8],
) -> IResult<&[u8], ConstState<W>, NomCustomError<&[u8]>> {
    let (i, final_weight) = W::parse_binary(i)?;
    let (i, pos) = parse_bin_i32(i)?;
    let (i, ntrs) = parse_bin_i32(i)?;
    let (i, niepsilons) = parse_bin_i32(i)?;
    let (i, noepsilons) = parse_bin_i32(i)?;

    Ok((
        i,
        ConstState {
            final_weight: parse_final_weight(final_weight),
            pos: pos as usize,
            ntrs: ntrs as usize,
            niepsilons: niepsilons as usize,
            noepsilons: noepsilons as usize,
        },
    ))
}

fn parse_const_fst<W: SerializableSemiring>(
    i: &[u8],
) -> IResult<&[u8], ConstFst<W>, NomCustomError<&[u8]>> {
    let stream_len = i.len();

    let (mut i, hdr) = FstHeader::parse(
        i,
        CONST_MIN_FILE_VERSION,
        ConstFst::<W>::fst_type(),
        Tr::<W>::tr_type(),
    )?;
    let aligned = hdr.version == CONST_ALIGNED_FILE_VERSION;
    let pos = stream_len - i.len();

    // Align input
    if aligned && hdr.num_states > 0 && pos % CONST_ARCH_ALIGNMENT > 0 {
        i = take(CONST_ARCH_ALIGNMENT - (pos % CONST_ARCH_ALIGNMENT))(i)?.0;
    }
    let (mut i, const_states) = count(parse_const_state, hdr.num_states as usize)(i)?;
    let pos = stream_len - i.len();

    // Align input
    if aligned && hdr.num_trs > 0 && pos % CONST_ARCH_ALIGNMENT > 0 {
        i = take(CONST_ARCH_ALIGNMENT - (pos % CONST_ARCH_ALIGNMENT))(i)?.0;
    }
    let (i, const_trs) = count(parse_bin_fst_tr, hdr.num_trs as usize)(i)?;

    Ok((
        i,
        ConstFst {
            start: parse_start_state(hdr.start),
            states: const_states,
            trs: Arc::new(const_trs),
            isymt: hdr.isymt,
            osymt: hdr.osymt,
            properties: FstProperties::from_bits_truncate(hdr.properties),
        },
    ))
}
