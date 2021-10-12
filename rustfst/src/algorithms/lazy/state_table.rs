use std::fmt;
use std::hash::Hash;
use std::sync::Mutex;

use crate::StateId;
use std::collections::hash_map::Entry;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::hash::BuildHasher;

use crate::parsers::nom_utils::NomCustomError;
use nom::multi::{count, fold_many_m_n};
use crate::algorithms::compose::filter_states::{FilterState, SerializableFilterState};
use nom::IResult;
use std::fs::{read, File};
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

use crate::parsers::bin_fst::utils_parsing::{
    parse_bin_u64
};
use crate::parsers::bin_fst::utils_serialization::{
   write_bin_u64
};

use anyhow::{anyhow, Context, Result};

#[derive(Clone, Debug, Default)]
pub(crate) struct BiHashMap<T: Hash + Eq + Clone, H: BuildHasher = RandomState> {
    tuple_to_id: HashMap<T, StateId, H>,
    id_to_tuple: Vec<T>,
}

impl<T: Hash + Eq + Clone, H: BuildHasher> PartialEq for BiHashMap<T, H> {
    fn eq(&self, other: &Self) -> bool {
        self.tuple_to_id.eq(&other.tuple_to_id) && self.id_to_tuple.eq(&other.id_to_tuple)
    }
}

impl<T: Hash + Eq + Clone> BiHashMap<T> {
    pub fn new() -> Self {
        Self {
            tuple_to_id: HashMap::new(),
            id_to_tuple: Vec::new(),
        }
    }
}

impl<T: Hash + Eq + Clone, H: BuildHasher> BiHashMap<T, H> {
    #[allow(unused)]
    pub fn with_hasher(hash_builder: H) -> Self {
        Self {
            tuple_to_id: HashMap::with_hasher(hash_builder),
            id_to_tuple: Vec::new(),
        }
    }

    pub fn get_id_or_insert(&mut self, tuple: T) -> StateId {
        match self.tuple_to_id.entry(tuple) {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(e) => {
                let n = self.id_to_tuple.len() as StateId;
                self.id_to_tuple.push(e.key().clone());
                e.insert(n);
                n
            }
        }
    }

    pub fn get_tuple_unchecked(&self, id: StateId) -> T {
        self.id_to_tuple[id as usize].clone()
    }
}

pub struct StateTable<T: Hash + Eq + Clone> {
    pub(crate) table: Mutex<BiHashMap<T>>,
}

impl<T: Hash + Eq + Clone> Clone for StateTable<T> {
    fn clone(&self) -> Self {
        Self {
            table: Mutex::new(self.table.lock().unwrap().clone()),
        }
    }
}

impl<T: Hash + Eq + Clone> Default for StateTable<T> {
    fn default() -> Self {
        Self {
            table: Mutex::new(BiHashMap::new()),
        }
    }
}

impl<T: Hash + Eq + Clone + fmt::Debug> fmt::Debug for StateTable<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StateTable {{ table : {:?} }}",
            self.table.lock().unwrap()
        )
    }
}

impl<T: Hash + Eq + Clone + PartialEq> PartialEq for StateTable<T> {
    fn eq(&self, other: &Self) -> bool {
        self.table.lock().unwrap().eq(&*other.table.lock().unwrap())
    }
}

impl<T: Hash + Eq + Clone> StateTable<T> {
    pub fn new() -> Self {
        Self {
            table: Mutex::new(BiHashMap::new()),
        }
    }

    /// Looks up integer ID from entry. If it doesn't exist and insert
    pub fn find_id_from_ref(&self, tuple: &T) -> StateId {
        let mut table = self.table.lock().unwrap();
        table.get_id_or_insert(tuple.clone())
    }

    pub fn find_id(&self, tuple: T) -> StateId {
        let mut table = self.table.lock().unwrap();
        table.get_id_or_insert(tuple)
    }

    /// Looks up tuple from integer ID.
    pub fn find_tuple(&self, tuple_id: StateId) -> T {
        let table = self.table.lock().unwrap();
        table.get_tuple_unchecked(tuple_id)
    }
}


impl<T: FilterState + SerializableFilterState> StateTable<T> {
    /// Loads a StateTable from a file in binary format.
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = read(path.as_ref())
            .with_context(|| format!("Can't open file : {:?}", path.as_ref()))?;

        // Parse StateTable
        let (_, state_table) = parse_state_table(&data)
            .map_err(|e| format_err!("Error while parsing binary StateTable : {:?}", e))?;

        Ok(state_table)
    }

    /// Writes a StateTable to a file in binary format.
    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut file = BufWriter::new(File::create(path)?);

        // Write StateTable
        write_state_table(&mut file, &self)?;

        Ok(())
    }
}

fn write_state_table<F: Write, T: FilterState + SerializableFilterState>(
    writter: &mut F,
    state_table: &StateTable<T>,
) -> Result<()> {
    let table = state_table.table.lock().map_err(|err| anyhow!("{}", err))?;
    write_bin_u64(writter, table.tuple_to_id.len() as u64)?;

    // Final weights serialization
    for (tuple, state) in table.tuple_to_id.iter()
    {
        (*tuple).write_binary(writter)?;
        write_bin_u64(writter, *state as u64)?;
    }

    write_bin_u64(writter, table.id_to_tuple.len() as u64)?;
    for tuple in table.id_to_tuple.iter() {
        (*tuple).write_binary(writter)?;
    }

    Ok(())
}

fn parse_state_table<T: FilterState + SerializableFilterState>(
    i: &[u8],
) -> IResult<&[u8], StateTable<T>, NomCustomError<&[u8]>> {
    let (i, tuple_to_id_len) = parse_bin_u64(i)?; 
    let (i, tuple_to_id) = fold_many_m_n(
        tuple_to_id_len as usize,
        tuple_to_id_len as usize,
        parse_tuple_to_id,
        HashMap::<T, StateId>::new(),
        |mut acc: HashMap<T, StateId>,
         item: (T, StateId)|
         -> HashMap<T, StateId> {
            acc.insert(item.0, item.1);
            acc
        },
    )(i)?;

    let (i, id_to_tuple_len) = parse_bin_u64(i)?;
    let (i, id_to_tuple) = count(T::parse_binary, id_to_tuple_len as usize)(i)?;
    Ok((i, StateTable{
        table: Mutex::new(
            BiHashMap {
                tuple_to_id,
                id_to_tuple,
            }
        )
    }))
}

fn parse_tuple_to_id<T: FilterState + SerializableFilterState>(
    i: &[u8],
) -> IResult<&[u8], (T, StateId), NomCustomError<&[u8]>> {
    let (i, tuple) = T::parse_binary(i)?;
    let (i, state) = parse_bin_u64(i)?;

    Ok((i, (tuple, state as StateId)))
}
