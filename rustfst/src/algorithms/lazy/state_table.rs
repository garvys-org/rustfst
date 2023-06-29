use std::fmt;
use std::hash::Hash;
use std::sync::Mutex;

use crate::StateId;
use std::collections::hash_map::Entry;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::hash::BuildHasher;

use crate::parsers::nom_utils::NomCustomError;
use crate::parsers::{parse_bin_u64, write_bin_u64, SerializeBinary};
use nom::multi::{count, fold_many_m_n};
use nom::IResult;
use std::io::Write;

use anyhow::{anyhow, Result};

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

impl<T: SerializeBinary + Hash + Eq + Clone> SerializeBinary for StateTable<T> {
    /// Parse a struct of type Self from a binary buffer.
    fn parse_binary(i: &[u8]) -> IResult<&[u8], Self, NomCustomError<&[u8]>> {
        let (i, tuple_to_id_len) = parse_bin_u64(i)?;
        let (i, tuple_to_id) = fold_many_m_n(
            tuple_to_id_len as usize,
            tuple_to_id_len as usize,
            parse_tuple_to_id,
            HashMap::<T, StateId>::new,
            |mut acc, item| {
                acc.insert(item.0, item.1);
                acc
            },
        )(i)?;

        let (i, id_to_tuple_len) = parse_bin_u64(i)?;
        let (i, id_to_tuple) = count(T::parse_binary, id_to_tuple_len as usize)(i)?;
        Ok((
            i,
            StateTable {
                table: Mutex::new(BiHashMap {
                    tuple_to_id,
                    id_to_tuple,
                }),
            },
        ))
    }
    /// Writes a struct to a writable buffer.
    fn write_binary<WB: Write>(&self, writer: &mut WB) -> Result<()> {
        let table = self.table.lock().map_err(|err| anyhow!("{}", err))?;
        write_bin_u64(writer, table.tuple_to_id.len() as u64)?;

        // Final weights serialization
        for (tuple, state) in table.tuple_to_id.iter() {
            (*tuple).write_binary(writer)?;
            write_bin_u64(writer, *state as u64)?;
        }

        write_bin_u64(writer, table.id_to_tuple.len() as u64)?;
        for tuple in table.id_to_tuple.iter() {
            (*tuple).write_binary(writer)?;
        }

        Ok(())
    }
}

fn parse_tuple_to_id<T: SerializeBinary>(
    i: &[u8],
) -> IResult<&[u8], (T, StateId), NomCustomError<&[u8]>> {
    let (i, tuple) = T::parse_binary(i)?;
    let (i, state) = parse_bin_u64(i)?;

    Ok((i, (tuple, state as StateId)))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::algorithms::compose::filter_states::{FilterState, IntegerFilterState};
    use crate::algorithms::compose::ComposeStateTuple;
    use crate::StateId;
    use anyhow::Result;

    #[test]
    fn test_read_write_state_table_empty() -> Result<()> {
        let state_table = StateTable::<ComposeStateTuple<IntegerFilterState>>::new();

        let mut buffer = Vec::new();
        state_table.write_binary(&mut buffer)?;
        let (_, parsed_state_table) =
            StateTable::<ComposeStateTuple<IntegerFilterState>>::parse_binary(&buffer)
                .map_err(|err| anyhow!("{}", err))?;

        assert_eq!(state_table, parsed_state_table);
        Ok(())
    }

    #[test]
    fn test_read_write_state_table() -> Result<()> {
        let fs1 = IntegerFilterState::new(1);
        let fs2 = IntegerFilterState::new(2);
        let tuple_1 = ComposeStateTuple {
            fs: fs1,
            s1: 1 as StateId,
            s2: 2 as StateId,
        };
        let tuple_2 = ComposeStateTuple {
            fs: fs2,
            s1: 1 as StateId,
            s2: 2 as StateId,
        };
        let state_table = StateTable::new();
        state_table.find_id(tuple_1);
        state_table.find_id(tuple_2);

        let mut buffer = Vec::new();
        state_table.write_binary(&mut buffer)?;
        let (_, parsed_state_table) =
            StateTable::<ComposeStateTuple<IntegerFilterState>>::parse_binary(&buffer)
                .map_err(|err| anyhow!("{}", err))?;

        assert_eq!(state_table, parsed_state_table);
        Ok(())
    }
}
