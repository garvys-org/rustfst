mod cache_internal_types;
pub mod cache_status;
pub mod first_cache;
pub mod fst_cache;
pub mod simple_hash_map_cache;
pub mod simple_vec_cache;
mod utils_parsing;
mod utils_serialization;

pub use self::cache_status::CacheStatus;
pub use self::first_cache::FirstCache;
pub use self::fst_cache::FstCache;
pub use self::simple_hash_map_cache::SimpleHashMapCache;
pub use self::simple_vec_cache::SimpleVecCache;

use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

/// Trait definining the methods a cache must implement to be serialized and deserialized.
pub trait SerializableCache {
    /// Loads a cache from a file in binary format.
    fn read<P: AsRef<Path>>(path: P) -> Result<Self>
    where
        Self: Sized;
    /// Writes a cache to a file in binary format.
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()>;
}

impl<C: SerializableCache> SerializableCache for Arc<C> {
    fn read<P: AsRef<Path>>(path: P) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Arc::new(C::read(path)?))
    }

    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        (**self).write(path)
    }
}
