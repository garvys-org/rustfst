use nom::number::complete::le_i32;
use nom::IResult;

use crate::parsers::nom_utils::NomCustomError;
use crate::semirings::SerializableSemiring;
use crate::{Label, StateId, Tr};

#[inline]
pub(crate) fn parse_start_state(s: i64) -> Option<StateId> {
    if s == -1 {
        None
    } else {
        Some(s as StateId)
    }
}

#[inline]
pub(crate) fn parse_final_weight<W: SerializableSemiring>(weight: W) -> Option<W> {
    // TODO: Avoid this re-allocation
    let zero_weight = W::zero();
    if weight != zero_weight {
        Some(weight)
    } else {
        None
    }
}

pub(crate) fn parse_bin_fst_tr<W: SerializableSemiring>(
    i: &[u8],
) -> IResult<&[u8], Tr<W>, NomCustomError<&[u8]>> {
    let (i, ilabel) = le_i32(i)?;
    let (i, olabel) = le_i32(i)?;
    let (i, weight) = W::parse_binary(i)?;
    let (i, nextstate) = le_i32(i)?;
    Ok((
        i,
        Tr {
            ilabel: ilabel as Label,
            olabel: olabel as Label,
            weight,
            nextstate: nextstate as StateId,
        },
    ))
}
