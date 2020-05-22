use std::fs::{read, File};
use std::io::BufWriter;
use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use itertools::Itertools;
use nom::bytes::complete::take;
use nom::multi::count;
use nom::number::complete::le_i32;
use nom::IResult;

use crate::fst_impls::const_fst::data_structure::ConstState;
use crate::fst_impls::ConstFst;
use crate::fst_traits::{ExpandedFst, Fst, SerializableFst};
use crate::parsers::bin_fst::fst_header::{FstFlags, FstHeader, OpenFstString, FST_MAGIC_NUMBER};
use crate::parsers::bin_fst::utils_parsing::{parse_final_weight, parse_fst_tr, parse_start_state};
use crate::parsers::bin_fst::utils_serialization::write_bin_i32;
use crate::parsers::text_fst::ParsedTextFst;
use crate::semirings::SerializableSemiring;
use crate::{Tr, EPS_LABEL};
use std::sync::Arc;

impl<W: 'static + SerializableSemiring> SerializableFst<W> for ConstFst<W> {
    fn fst_type() -> String {
        "const".to_string()
    }

    fn read<P: AsRef<Path>>(path_bin_fst: P) -> Result<Self> {
        let data = read(path_bin_fst.as_ref()).with_context(|| {
            format!(
                "Can't open ConstFst binary file : {:?}",
                path_bin_fst.as_ref()
            )
        })?;

        let (_, parsed_fst) = parse_const_fst(&data)
            .map_err(|_| format_err!("Error while parsing binary ConstFst"))?;

        Ok(parsed_fst)
    }

    fn write<P: AsRef<Path>>(&self, path_bin_fst: P) -> Result<()> {
        let mut file = BufWriter::new(File::create(path_bin_fst)?);

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
            // TODO: Once the properties are stored, need to read them. kExpanded
            properties: 1u64,
            start: self.start.map(|v| v as i64).unwrap_or(-1),
            num_states: self.num_states() as i64,
            num_trs: self.trs.len() as i64,
            isymt: self.input_symbols().cloned(),
            osymt: self.output_symbols().cloned(),
        };
        hdr.write(&mut file)?;

        let zero = W::zero();
        for const_state in &self.states {
            let f_weight = const_state.final_weight.as_ref().unwrap_or_else(|| &zero);
            f_weight.write_binary(&mut file)?;

            write_bin_i32(&mut file, const_state.pos as i32)?;
            write_bin_i32(&mut file, const_state.ntrs as i32)?;
            write_bin_i32(&mut file, const_state.niepsilons as i32)?;
            write_bin_i32(&mut file, const_state.noepsilons as i32)?;
        }

        for tr in &*self.trs {
            write_bin_i32(&mut file, tr.ilabel as i32)?;
            write_bin_i32(&mut file, tr.olabel as i32)?;
            tr.weight.write_binary(&mut file)?;
            write_bin_i32(&mut file, tr.nextstate as i32)?;
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
            const_states.resize_with(_state, || ConstState {
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
                    .get_unchecked_mut(final_state.state)
                    .final_weight = Some(weight)
            };
        }

        Ok(ConstFst {
            states: const_states,
            trs: Arc::new(const_trs),
            start: start_state,
            isymt: None,
            osymt: None,
        })
    }
}

static CONST_MIN_FILE_VERSION: i32 = 1;
static CONST_ALIGNED_FILE_VERSION: i32 = 1;
static CONST_FILE_VERSION: i32 = 2;
static CONST_ARCH_ALIGNMENT: usize = 16;

fn parse_const_state<W: SerializableSemiring>(i: &[u8]) -> IResult<&[u8], ConstState<W>> {
    let (i, final_weight) = W::parse_binary(i)?;
    let (i, pos) = le_i32(i)?;
    let (i, ntrs) = le_i32(i)?;
    let (i, niepsilons) = le_i32(i)?;
    let (i, noepsilons) = le_i32(i)?;

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

fn parse_const_fst<W: SerializableSemiring + 'static>(i: &[u8]) -> IResult<&[u8], ConstFst<W>> {
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
    let (i, const_trs) = count(parse_fst_tr, hdr.num_trs as usize)(i)?;

    Ok((
        i,
        ConstFst {
            start: parse_start_state(hdr.start),
            states: const_states,
            trs: Arc::new(const_trs),
            isymt: hdr.isymt,
            osymt: hdr.osymt,
        },
    ))
}
