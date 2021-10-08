use std::collections::HashMap;
use std::fs::{read, File};
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::parsers::nom_utils::NomCustomError;
use anyhow::{anyhow, format_err, Context, Result};
use nom::multi::{count, fold_many_m_n};
use nom::IResult;

use super::SerializableCache;
use crate::algorithms::lazy::cache::cache_internal_types::{
    CacheTrs, CachedData, FinalWeight, StartState,
};
use crate::algorithms::lazy::{CacheStatus, FstCache};
use crate::parsers::bin_fst::utils_parsing::{
    parse_bin_fst_tr, parse_bin_i64, parse_bin_u64, parse_bin_u8, parse_final_weight,
    parse_start_state,
};
use crate::parsers::bin_fst::utils_serialization::{
    write_bin_fst_tr, write_bin_i64, write_bin_u64, write_bin_u8, write_final_weight,
};
use crate::semirings::{Semiring, SerializableSemiring};
use crate::{StateId, Trs, TrsVec, EPS_LABEL};

#[derive(Debug)]
pub struct SimpleHashMapCache<W: Semiring> {
    // First option : has start been computed
    // Second option: value of the start state (possibly none)
    // The second element of each tuple is the number of known states.
    start: Mutex<CachedData<CacheStatus<StartState>>>,
    trs: Mutex<CachedData<HashMap<StateId, CacheTrs<W>>>>,
    final_weights: Mutex<CachedData<HashMap<StateId, Option<W>>>>,
}

impl<W: Semiring> SimpleHashMapCache<W> {
    pub fn clear(&self) {
        let mut data_start = self.start.lock().unwrap();
        data_start.clear();

        let mut data_trs = self.trs.lock().unwrap();
        data_trs.clear();

        let mut data_final_weights = self.final_weights.lock().unwrap();
        data_final_weights.clear();
    }
}

impl<W: Semiring> Clone for SimpleHashMapCache<W> {
    fn clone(&self) -> Self {
        Self {
            start: Mutex::new(self.start.lock().unwrap().clone()),
            trs: Mutex::new(self.trs.lock().unwrap().clone()),
            final_weights: Mutex::new(self.final_weights.lock().unwrap().clone()),
        }
    }
}

impl<W: Semiring> Default for SimpleHashMapCache<W> {
    fn default() -> Self {
        Self {
            start: Mutex::new(CachedData::default()),
            trs: Mutex::new(CachedData::default()),
            final_weights: Mutex::new(CachedData::default()),
        }
    }
}

impl<W: Semiring> FstCache<W> for SimpleHashMapCache<W> {
    fn get_start(&self) -> CacheStatus<Option<StateId>> {
        let res = self.start.lock().unwrap();
        res.data
    }

    fn insert_start(&self, id: Option<StateId>) {
        let mut data = self.start.lock().unwrap();
        if let Some(s) = id {
            data.num_known_states = std::cmp::max(data.num_known_states, s as usize + 1);
        }
        data.data = CacheStatus::Computed(id);
    }

    fn get_trs(&self, id: StateId) -> CacheStatus<TrsVec<W>> {
        match self.trs.lock().unwrap().get(id) {
            Some(e) => CacheStatus::Computed(e.trs.shallow_clone()),
            None => CacheStatus::NotComputed,
        }
    }

    fn insert_trs(&self, id: StateId, trs: TrsVec<W>) {
        let mut cached_data = self.trs.lock().unwrap();
        let mut niepsilons = 0;
        let mut noepsilons = 0;
        for tr in trs.trs() {
            cached_data.num_known_states =
                std::cmp::max(cached_data.num_known_states, tr.nextstate as usize + 1);
            if tr.ilabel == EPS_LABEL {
                niepsilons += 1;
            }
            if tr.olabel == EPS_LABEL {
                noepsilons += 1;
            }
        }
        cached_data.data.insert(
            id,
            CacheTrs {
                trs,
                niepsilons,
                noepsilons,
            },
        );
    }

    fn compute_num_known_trs(&self) -> usize {
        let cached_data = self.trs.lock().unwrap();
        cached_data.data.values().map(|it| it.trs.trs().len()).sum()
    }

    fn get_final_weight(&self, id: StateId) -> CacheStatus<Option<W>> {
        match self.final_weights.lock().unwrap().get(id) {
            Some(e) => CacheStatus::Computed(e.clone()),
            None => CacheStatus::NotComputed,
        }
    }

    fn insert_final_weight(&self, id: StateId, weight: Option<W>) {
        let mut cached_data = self.final_weights.lock().unwrap();
        cached_data.num_known_states = std::cmp::max(cached_data.num_known_states, id as usize + 1);
        cached_data.data.insert(id, weight);
    }

    fn num_known_states(&self) -> usize {
        let mut n = 0;
        n = std::cmp::max(n, self.start.lock().unwrap().num_known_states);
        n = std::cmp::max(n, self.trs.lock().unwrap().num_known_states);
        n = std::cmp::max(n, self.final_weights.lock().unwrap().num_known_states);
        n
    }

    fn num_trs(&self, id: StateId) -> Option<usize> {
        let cached_data = self.trs.lock().unwrap();
        cached_data.get(id).map(|v| v.trs.len())
    }

    fn num_input_epsilons(&self, id: StateId) -> Option<usize> {
        let cached_data = self.trs.lock().unwrap();
        cached_data.get(id).map(|v| v.niepsilons)
    }

    fn num_output_epsilons(&self, id: StateId) -> Option<usize> {
        let cached_data = self.trs.lock().unwrap();
        cached_data.get(id).map(|v| v.noepsilons)
    }

    fn len_trs(&self) -> usize {
        let cached_data = self.trs.lock().unwrap();
        cached_data.data.len()
    }

    fn len_final_weights(&self) -> usize {
        let cached_data = self.final_weights.lock().unwrap();
        cached_data.data.len()
    }
}

impl<W: SerializableSemiring> SerializableCache for SimpleHashMapCache<W> {
    /// Loads a SimpleHashMapCache from a file in binary format.
    fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = read(path.as_ref())
            .with_context(|| format!("Can't open file : {:?}", path.as_ref()))?;

        // Parse SimpleHashMapCache
        let (_, simple_vec_cache) = parse_simple_hashmap_cache(&data)
            .map_err(|e| format_err!("Error while parsing binary SimpleHashMapCache : {:?}", e))?;

        Ok(simple_vec_cache)
    }

    /// Writes a SimpleHashMapCache to a file in binary format.
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut file = BufWriter::new(File::create(path)?);

        // Serialize SimpleHashMapCache
        serialize_simple_hashmap_cache(&self, &mut file)?;

        Ok(())
    }
}

pub fn serialize_simple_hashmap_cache<F: Write, W: SerializableSemiring>(
    cache: &SimpleHashMapCache<W>,
    writter: &mut F,
) -> Result<()> {
    // Num known states serialization
    let num_known_states = cache.num_known_states();
    write_bin_u64(writter, num_known_states as u64)?;

    // Start state serialization
    write_cache_start_state(writter, &cache.get_start())?;

    // Num computed states
    let num_visited_states = cache.len_trs();
    write_bin_u64(writter, num_visited_states as u64)?;

    // Computed states serialization
    for (state, cache_trs) in cache
        .trs
        .lock()
        .map_err(|err| anyhow!("{}", err))?
        .data
        .iter()
    {
        // Write CacheTrs trs
        write_cache_trs(writter, cache_trs, state)?;
    }

    // Num computed final weights
    let num_final_weights = cache.len_final_weights();
    write_bin_u64(writter, num_final_weights as u64)?;

    // Final weights serialization
    for (state, final_weight) in cache
        .final_weights
        .lock()
        .map_err(|err| anyhow!("{}", err))?
        .data
        .iter()
    {
        // Write final weight for state
        write_cache_final_weight(writter, final_weight, state)?;
    }
    Ok(())
}

fn write_cache_start_state<F: Write>(
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

fn write_cache_trs<F: Write, W: SerializableSemiring>(
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

fn write_cache_final_weight<F: Write, W: SerializableSemiring>(
    writter: &mut F,
    final_weight: &FinalWeight<W>,
    state: &StateId,
) -> Result<()> {
    // Write state
    write_bin_u64(writter, *state as u64)?;
    write_final_weight(writter, final_weight)?;
    Ok(())
}

pub fn parse_simple_hashmap_cache<W: SerializableSemiring>(
    i: &[u8],
) -> IResult<&[u8], SimpleHashMapCache<W>, NomCustomError<&[u8]>> {
    // Parse num known states
    let (i, num_known_states) = parse_bin_u64(i)?;

    // Parse start node
    let (i, start_node) = parse_cache_start_state(i)?;

    // Parse num computed states
    let (i, num_computed_states) = parse_bin_u64(i)?;

    let (i, trs_data) = fold_many_m_n(
        num_computed_states as usize,
        num_computed_states as usize,
        parse_hashmap_cache_trs::<W>,
        HashMap::<StateId, CacheTrs<W>>::new(),
        |mut acc: HashMap<StateId, CacheTrs<W>>,
         item: (StateId, CacheTrs<W>)|
         -> HashMap<StateId, CacheTrs<W>> {
            acc.insert(item.0, item.1);
            acc
        },
    )(i)?;

    // Parse num computed final weights
    let (i, num_computed_final_weights) = parse_bin_u64(i)?;

    let (i, final_weights_data) = fold_many_m_n(
        num_computed_final_weights as usize,
        num_computed_final_weights as usize,
        parse_cache_final_weight::<W>,
        HashMap::<StateId, FinalWeight<W>>::new(),
        |mut acc: HashMap<StateId, FinalWeight<W>>,
         item: (StateId, FinalWeight<W>)|
         -> HashMap<StateId, FinalWeight<W>> {
            acc.insert(item.0, item.1);
            acc
        },
    )(i)?;

    Ok((
        i,
        SimpleHashMapCache {
            start: Mutex::new(CachedData {
                data: start_node,
                num_known_states: num_known_states as usize,
            }),
            trs: Mutex::new(CachedData {
                data: trs_data,
                num_known_states: num_known_states as usize,
            }),
            final_weights: Mutex::new(CachedData {
                data: final_weights_data,
                num_known_states: num_known_states as usize,
            }),
        },
    ))
}

fn parse_hashmap_cache_trs<W: SerializableSemiring>(
    i: &[u8],
) -> IResult<&[u8], (StateId, CacheTrs<W>), NomCustomError<&[u8]>> {
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

fn parse_cache_final_weight<W: SerializableSemiring>(
    i: &[u8],
) -> IResult<&[u8], (StateId, FinalWeight<W>), NomCustomError<&[u8]>> {
    let (i, state) = parse_bin_i64(i)?;
    let (i, raw_final_weight) = W::parse_binary(i)?;
    Ok((i, (state as StateId, parse_final_weight(raw_final_weight))))
}

fn parse_cache_start_state(
    i: &[u8],
) -> IResult<&[u8], CacheStatus<StartState>, NomCustomError<&[u8]>> {
    let (i, is_computed) = parse_bin_u8(i)?;
    if is_computed == 0 {
        Ok((i, CacheStatus::NotComputed))
    } else {
        let (i, raw_start_state) = parse_bin_i64(i)?;
        Ok((i, CacheStatus::Computed(parse_start_state(raw_start_state))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::Tr;
    use crate::semirings::TropicalWeight;
    use anyhow::anyhow;

    #[test]
    fn test_read_write_cache_start_state_computed() -> Result<()> {
        let cache_start_state = CacheStatus::Computed(StartState::default());
        let mut buffer = Vec::new();
        write_cache_start_state(&mut buffer, &cache_start_state)?;
        let (_, parsed_cache_start_state) =
            parse_cache_start_state(&buffer).map_err(|err| anyhow!("{}", err))?;
        assert_eq!(cache_start_state, parsed_cache_start_state);
        Ok(())
    }

    #[test]
    fn test_read_write_cache_start_state_not_computed() -> Result<()> {
        let cache_start_state = CacheStatus::NotComputed;
        let mut buffer = Vec::new();
        write_cache_start_state(&mut buffer, &cache_start_state)?;
        let (_, parsed_cache_start_state) =
            parse_cache_start_state(&buffer).map_err(|err| anyhow!("{}", err))?;
        assert_eq!(cache_start_state, parsed_cache_start_state);
        Ok(())
    }

    #[test]
    fn test_read_write_cache_final_weight() -> Result<()> {
        let final_weight = FinalWeight::<TropicalWeight>::default();
        let mut buffer = Vec::new();
        write_cache_final_weight(&mut buffer, &final_weight, &0)?;
        let (_, (parsed_state, parsed_final_weight)) =
            parse_cache_final_weight(&buffer).map_err(|err| anyhow!("{}", err))?;
        assert_eq!(final_weight, parsed_final_weight);
        assert_eq!(0, parsed_state);
        Ok(())
    }

    #[test]
    fn test_read_write_cache_trs() -> Result<()> {
        let mut trs = TrsVec::<TropicalWeight>::default();
        trs.push(Tr::new(0, 1, TropicalWeight::one(), 2));
        trs.push(Tr::new(0, 1, TropicalWeight::one(), 0));
        trs.push(Tr::new(0, 1, TropicalWeight::zero(), 10));
        let cache_trs = CacheTrs {
            trs,
            niepsilons: 3,
            noepsilons: 2,
        };
        let mut buffer = Vec::new();
        write_cache_trs(&mut buffer, &cache_trs, &0)?;
        let (_, (parsed_state, parsed_cache_trs)) =
            parse_hashmap_cache_trs(&buffer).map_err(|err| anyhow!("{}", err))?;
        assert_eq!(cache_trs, parsed_cache_trs);
        assert_eq!(0, parsed_state);
        Ok(())
    }
}
