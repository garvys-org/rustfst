use std::io::Write;
use std::sync::Arc;

use anyhow::Result;
use nom::multi::count;
use nom::number::complete::le_i64;
use nom::IResult;

use crate::fst_impls::vector_fst::VectorFstState;
use crate::fst_impls::VectorFst;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{CoreFst, ExpandedFst, Fst, MutableFst, SerializableFst};
use crate::parsers::bin_fst::fst_header::{FstFlags, FstHeader, OpenFstString, FST_MAGIC_NUMBER};
use crate::parsers::bin_fst::utils_parsing::{
    parse_bin_fst_tr, parse_final_weight, parse_start_state,
};
use crate::parsers::bin_fst::utils_serialization::write_bin_fst_tr;
use crate::parsers::nom_utils::NomCustomError;
use crate::parsers::text_fst::ParsedTextFst;
use crate::parsers::write_bin_i64;
use crate::semirings::SerializableSemiring;
use crate::{StateId, Tr, Trs, TrsVec, EPS_LABEL};

impl<W: SerializableSemiring> SerializableFst<W> for VectorFst<W> {
    fn fst_type() -> String {
        "vector".to_string()
    }

    fn load(data: &[u8]) -> Result<Self> {
        let (_, parsed_fst) = parse_vector_fst(data).map_err(|e| {
            e.map(|e_inner| match e_inner {
                NomCustomError::Nom(_, k) => {
                    format_err!("Error while parsing binary VectorFst. Error kind {:?}", k)
                }
                NomCustomError::SymbolTableError(e) => format_err!(
                    "Error while parsing symbolTable from binary VectorFst : {}",
                    e
                ),
            })
        })?;

        Ok(parsed_fst)
    }

    fn store<O: Write>(&self, mut output: O) -> Result<()> {
        let num_trs: usize = (0..self.num_states())
            .map(|s: usize| unsafe { self.num_trs_unchecked(s as StateId) })
            .sum();

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
            version: 2i32,
            // TODO: Set flags if the content is aligned
            flags,
            // Static properties are added to the property bits to be compliant with OpenFst format.
            properties: self.properties.bits() | VectorFst::<W>::static_properties(),
            start: self.start_state.map(|v| v as i64).unwrap_or(-1),
            num_states: self.num_states() as i64,
            num_trs: num_trs as i64,
            isymt: self.input_symbols().cloned(),
            osymt: self.output_symbols().cloned(),
        };
        hdr.write(&mut output)?;

        // FstBody
        for state in 0..self.num_states() {
            let state = state as StateId;
            let f_weight = unsafe { self.final_weight_unchecked(state).unwrap_or_else(W::zero) };
            f_weight.write_binary(&mut output)?;
            write_bin_i64(&mut output, unsafe { self.num_trs_unchecked(state) } as i64)?;

            for tr in unsafe { self.get_trs_unchecked(state).trs() } {
                write_bin_fst_tr(&mut output, tr)?;
            }
        }

        Ok(())
    }

    fn from_parsed_fst_text(parsed_fst_text: ParsedTextFst<W>) -> Result<Self> {
        let start_state = parsed_fst_text.start();
        let num_states = parsed_fst_text.num_states();

        let states = vec![VectorFstState::<W>::new(); num_states];

        let mut fst = VectorFst {
            states,
            start_state,
            isymt: None,
            osymt: None,
            properties: FstProperties::empty(),
        };

        for transition in parsed_fst_text.transitions.into_iter() {
            let weight = transition.weight.unwrap_or_else(W::one);
            let tr = Tr::new(
                transition.ilabel,
                transition.olabel,
                weight,
                transition.nextstate,
            );
            fst.add_tr(transition.state, tr)?;
        }

        for final_state in parsed_fst_text.final_states.into_iter() {
            let weight = final_state.weight.unwrap_or_else(W::one);
            fst.set_final(final_state.state, weight)?;
        }

        // Compute properties. Should be moved elsewhere
        fst.compute_and_update_properties_all()?;

        Ok(fst)
    }
}

static VECTOR_MIN_FILE_VERSION: i32 = 2;

fn parse_vector_fst_state<W: SerializableSemiring>(
    i: &[u8],
) -> IResult<&[u8], VectorFstState<W>, NomCustomError<&[u8]>> {
    let (i, final_weight) = W::parse_binary(i)?;
    let (i, num_trs) = le_i64(i)?;
    let (i, trs) = count(parse_bin_fst_tr, num_trs as usize)(i)?;
    let niepsilons = trs.iter().filter(|t| t.ilabel == EPS_LABEL).count();
    let noepsilons = trs.iter().filter(|t| t.olabel == EPS_LABEL).count();
    Ok((
        i,
        VectorFstState {
            final_weight: parse_final_weight(final_weight),
            trs: TrsVec(Arc::new(trs)),
            niepsilons,
            noepsilons,
        },
    ))
}

fn parse_vector_fst<W: SerializableSemiring>(
    i: &[u8],
) -> IResult<&[u8], VectorFst<W>, NomCustomError<&[u8]>> {
    let (i, header) = FstHeader::parse(
        i,
        VECTOR_MIN_FILE_VERSION,
        VectorFst::<W>::fst_type(),
        Tr::<W>::tr_type(),
    )?;
    let (i, states) = count(parse_vector_fst_state, header.num_states as usize)(i)?;
    Ok((
        i,
        VectorFst {
            start_state: parse_start_state(header.start),
            states,
            isymt: header.isymt,
            osymt: header.osymt,
            properties: FstProperties::from_bits_truncate(header.properties),
        },
    ))
}
