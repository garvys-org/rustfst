use nom::number::complete::le_i32;
use nom::IResult;

use crate::semirings::SerializableSemiring;
use crate::Arc;
use crate::StateId;
use crate::NO_STATE_ID;

#[inline]
pub(crate) fn parse_start_state(s: i64) -> Option<StateId> {
    if s == (NO_STATE_ID as i64) {
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

pub(crate) fn parse_fst_arc<W: SerializableSemiring>(i: &[u8]) -> IResult<&[u8], Arc<W>> {
    let (i, ilabel) = le_i32(i)?;
    let (i, olabel) = le_i32(i)?;
    let (i, weight) = W::parse_binary(i)?;
    let (i, nextstate) = le_i32(i)?;
    Ok((
        i,
        Arc {
            ilabel: ilabel as usize,
            olabel: olabel as usize,
            weight,
            nextstate: nextstate as usize,
        },
    ))
}
