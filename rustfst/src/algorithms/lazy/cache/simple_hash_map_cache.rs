use std::collections::HashMap;
use std::fs::{read, File};
use std::io::BufWriter;
use std::path::Path;
use std::sync::Mutex;

use anyhow::{anyhow, format_err, Context, Result};
use nom::IResult;

use super::SerializableCache;
use crate::algorithms::lazy::cache::cache_internal_types::{CacheTrs, CachedData, StartState};
use crate::algorithms::lazy::{CacheStatus, FstCache};
use crate::parsers::bin_fst::utils_serialization::{write_bin_i32, write_bin_i64, write_bin_u64};
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
        match self.trs.lock().unwrap().data.get(&id) {
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
        match self.final_weights.lock().unwrap().data.get(&id) {
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
        cached_data.data.get(&id).map(|v| v.trs.len())
    }

    fn num_input_epsilons(&self, id: StateId) -> Option<usize> {
        let cached_data = self.trs.lock().unwrap();
        cached_data.data.get(&id).map(|v| v.niepsilons)
    }

    fn num_output_epsilons(&self, id: StateId) -> Option<usize> {
        let cached_data = self.trs.lock().unwrap();
        cached_data.data.get(&id).map(|v| v.noepsilons)
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

        // Start state serialization
        match self.get_start() {
            CacheStatus::Computed(v) => {
                write_bin_i64(&mut file, v.map_or_else(|| -1, |v| v as i64))?
            }
            CacheStatus::NotComputed => write_bin_i64(&mut file, -2)?,
        };

        // Trs & final states serialization
        // TODO: avoid loop accross all known states -> Need to handle properly final states
        let num_states = self.num_known_states();
        write_bin_u64(&mut file, num_states as u64)?;
        for state in 0..num_states {
            let state = state as StateId;
            if let Some(cache_trs) = self
                .trs
                .lock()
                .map_err(|err| anyhow!("{}", err))?
                .get(state)
            {
                // Write state (non written states are NotComputed)
                write_bin_u64(&mut file, state as u64)?;

                // Write final weight for state (not written final weights are NotComputed)
                match &self.get_final_weight(state) {
                    CacheStatus::Computed(final_weight) => {
                        if let Some(final_weight) = final_weight {
                            final_weight.write_binary(&mut file)?
                        } else {
                            write_bin_i64(&mut file, -1)?
                        }
                    }
                    CacheStatus::NotComputed => write_bin_i64(&mut file, -2)?,
                };

                // Write CacheTrs trs
                write_bin_u64(&mut file, cache_trs.trs.len() as u64)?;
                for tr in cache_trs.trs.iter() {
                    write_bin_i32(&mut file, tr.ilabel as i32)?;
                    write_bin_i32(&mut file, tr.olabel as i32)?;
                    tr.weight.write_binary(&mut file)?;
                    write_bin_i32(&mut file, tr.nextstate as i32)?;
                }

                // Write CacheTrs niepsilons
                write_bin_u64(&mut file, cache_trs.niepsilons as u64)?;

                // Write CacheTrs noepsilons
                write_bin_u64(&mut file, cache_trs.noepsilons as u64)?;
            }
        }
        Ok(())
    }
}

pub fn parse_simple_hashmap_cache<W: SerializableSemiring>(
    i: &[u8],
) -> IResult<&[u8], SimpleHashMapCache<W>> {
    unimplemented!()
}
