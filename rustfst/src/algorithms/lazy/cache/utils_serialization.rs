use anyhow::Result;

use std::io::Write;

use crate::algorithms::lazy::cache::cache_internal_types::{CacheTrs, FinalWeight, StartState};
use crate::algorithms::lazy::CacheStatus;
use crate::parsers::bin_fst::utils_serialization::{write_bin_fst_tr, write_final_weight};
use crate::parsers::{write_bin_i64, write_bin_u64, write_bin_u8};
use crate::prelude::{SerializableSemiring, StateId};

pub(crate) fn write_cache_start_state<F: Write>(
    writter: &mut F,
    cache_start_state: &CacheStatus<StartState>,
) -> Result<()> {
    match cache_start_state {
        CacheStatus::Computed(v) => {
            // Mark as computed
            write_bin_u8(writter, 1)?;
            write_bin_i64(writter, v.map_or_else(|| -1, |v| v as i64))?;
        }
        CacheStatus::NotComputed => {
            // Mark state as NotComputed
            write_bin_u8(writter, 0)?;
        }
    }
    Ok(())
}

pub(crate) fn write_vec_cache_trs<F: Write, W: SerializableSemiring>(
    writter: &mut F,
    cache_trs: &CacheStatus<&CacheTrs<W>>,
) -> Result<()> {
    match cache_trs {
        CacheStatus::Computed(cache_trs) => {
            // Mark as computed
            write_bin_u8(writter, 1)?;
            // Write CacheTrs trs
            write_bin_u64(writter, cache_trs.trs.len() as u64)?;
            for tr in cache_trs.trs.iter() {
                write_bin_fst_tr(writter, tr)?;
            }
            // Write CacheTrs niepsilons
            write_bin_u64(writter, cache_trs.niepsilons as u64)?;
            // Write CacheTrs noepsilons
            write_bin_u64(writter, cache_trs.noepsilons as u64)?;
        }
        CacheStatus::NotComputed => {
            // Mark as NotComputed
            write_bin_u8(writter, 0)?;
        }
    }
    Ok(())
}

pub(crate) fn write_hashmap_cache_trs<F: Write, W: SerializableSemiring>(
    writter: &mut F,
    cache_trs: &CacheTrs<W>,
    state: &StateId,
) -> Result<()> {
    // Write state
    write_bin_u64(writter, *state as u64)?;

    // Write CacheTrs trs
    write_bin_u64(writter, cache_trs.trs.len() as u64)?;
    for tr in cache_trs.trs.iter() {
        write_bin_fst_tr(writter, tr)?;
    }
    // Write CacheTrs niepsilons
    write_bin_u64(writter, cache_trs.niepsilons as u64)?;
    // Write CacheTrs noepsilons
    write_bin_u64(writter, cache_trs.noepsilons as u64)?;

    Ok(())
}

pub(crate) fn write_vec_cache_final_weight<F: Write, W: SerializableSemiring>(
    writter: &mut F,
    cache_final_weight: &CacheStatus<FinalWeight<W>>,
) -> Result<()> {
    match cache_final_weight {
        CacheStatus::Computed(final_weight) => {
            // Mark as Computed
            write_bin_u8(writter, 1)?;
            write_final_weight(writter, final_weight)?;
        }
        CacheStatus::NotComputed => {
            // Mark as NotComputed
            write_bin_u8(writter, 0)?;
        }
    }

    Ok(())
}

pub(crate) fn write_hashmap_cache_final_weight<F: Write, W: SerializableSemiring>(
    writter: &mut F,
    final_weight: &FinalWeight<W>,
    state: &StateId,
) -> Result<()> {
    // Write state
    write_bin_u64(writter, *state as u64)?;
    write_final_weight(writter, final_weight)?;
    Ok(())
}
