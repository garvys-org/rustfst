use nom::number::complete::{le_f32, le_i32};
use nom::IResult;

use crate::semirings::Semiring;
use crate::Arc;
use crate::StateId;
use crate::NO_STATE_ID;

#[inline]
pub(crate) fn parse_start_state(s: i64) -> Option<StateId> {
    if s == i64::from(NO_STATE_ID) {
        None
    } else {
        Some(s as StateId)
    }
}

#[inline]
pub(crate) fn parse_final_weight<W: Semiring<Type = f32>>(w: f32) -> Option<W> {
    // TODO: Avoid this re-allocation
    let zero_weight = W::zero().take_value();
    if w != zero_weight {
        Some(W::new(w))
    } else {
        None
    }
}

pub(crate) fn parse_fst_arc<W: Semiring<Type = f32>>(i: &[u8]) -> IResult<&[u8], Arc<W>> {
    let (i, ilabel) = le_i32(i)?;
    let (i, olabel) = le_i32(i)?;
    let (i, weight) = le_f32(i)?;
    let (i, nextstate) = le_i32(i)?;
    Ok((
        i,
        Arc {
            ilabel: ilabel as usize,
            olabel: olabel as usize,
            weight: W::new(weight),
            nextstate: nextstate as usize,
        },
    ))
}
