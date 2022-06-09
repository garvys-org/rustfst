use std::sync::Arc;

use nom::multi::count;
use nom::IResult;

use crate::algorithms::lazy::cache::cache_internal_types::{CacheTrs, FinalWeight, StartState};
use crate::algorithms::lazy::CacheStatus;
use crate::parsers::bin_fst::utils_parsing::{parse_bin_fst_tr, parse_start_state};
use crate::parsers::nom_utils::NomCustomError;
use crate::parsers::{parse_bin_i64, parse_bin_u64, parse_bin_u8};
use crate::prelude::{SerializableSemiring, StateId, TrsVec};

pub(crate) type IResultCustomError<A, B> = IResult<A, B, NomCustomError<A>>;

pub(crate) fn parse_cache_start_state(
    i: &[u8],
) -> IResultCustomError<&[u8], CacheStatus<StartState>> {
    let (i, is_computed) = parse_bin_u8(i)?;
    if is_computed == 0 {
        Ok((i, CacheStatus::NotComputed))
    } else {
        let (i, raw_start_state) = parse_bin_i64(i)?;
        Ok((i, CacheStatus::Computed(parse_start_state(raw_start_state))))
    }
}

pub(crate) fn parse_vec_cache_trs<W: SerializableSemiring>(
    i: &[u8],
) -> IResultCustomError<&[u8], CacheStatus<CacheTrs<W>>> {
    let (i, is_computed) = parse_bin_u8(i)?;

    if is_computed == 0 {
        Ok((i, CacheStatus::NotComputed))
    } else {
        let (i, num_trs) = parse_bin_i64(i)?;
        let (i, trs) = count(parse_bin_fst_tr::<W>, num_trs as usize)(i)?;
        let (i, niepsilons) = parse_bin_u64(i)?;
        let (i, noepsilons) = parse_bin_u64(i)?;

        Ok((
            i,
            CacheStatus::Computed(CacheTrs {
                trs: TrsVec(Arc::new(trs)),
                niepsilons: niepsilons as usize,
                noepsilons: noepsilons as usize,
            }),
        ))
    }
}

pub(crate) fn parse_hashmap_cache_trs<W: SerializableSemiring>(
    i: &[u8],
) -> IResultCustomError<&[u8], (StateId, CacheTrs<W>)> {
    let (i, state) = parse_bin_i64(i)?;
    let (i, num_trs) = parse_bin_i64(i)?;
    let (i, trs) = count(parse_bin_fst_tr::<W>, num_trs as usize)(i)?;
    let (i, niepsilons) = parse_bin_u64(i)?;
    let (i, noepsilons) = parse_bin_u64(i)?;

    Ok((
        i,
        (
            state as StateId,
            CacheTrs {
                trs: TrsVec(Arc::new(trs)),
                niepsilons: niepsilons as usize,
                noepsilons: noepsilons as usize,
            },
        ),
    ))
}

pub(crate) fn parse_vec_cache_final_weight<W: SerializableSemiring>(
    i: &[u8],
) -> IResultCustomError<&[u8], CacheStatus<FinalWeight<W>>> {
    let (i, is_computed) = parse_bin_u8(i)?;

    if is_computed == 0 {
        Ok((i, CacheStatus::NotComputed))
    } else {
        let (i, is_some) = parse_bin_u8(i)?;
        if is_some == 1 {
            let (i, final_weight) = W::parse_binary(i)?;
            Ok((i, CacheStatus::Computed(Some(final_weight))))
        } else {
            Ok((i, CacheStatus::Computed(None)))
        }
    }
}

pub(crate) fn parse_hashmap_cache_final_weight<W: SerializableSemiring>(
    i: &[u8],
) -> IResultCustomError<&[u8], (StateId, FinalWeight<W>)> {
    let (i, state) = parse_bin_i64(i)?;
    let (i, is_some) = parse_bin_u8(i)?;
    if is_some == 1 {
        let (i, final_weight) = W::parse_binary(i)?;
        Ok((i, (state as StateId, Some(final_weight))))
    } else {
        Ok((i, (state as StateId, None)))
    }
}
