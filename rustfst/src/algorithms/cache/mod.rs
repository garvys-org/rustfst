use bitflags::bitflags;

pub use self::cache_impl::CacheImpl;
pub use self::cache_state::CacheState;
pub use self::fst_impl::FstImpl;
pub use self::state_table::StateTable;
pub use self::vector_cache_state::VectorCacheState;

mod cache_impl;
mod cache_state;
mod fst_impl;
mod state_table;
mod vector_cache_state;

bitflags! {
    pub struct CacheFlags: u32 {
        // Final weight has been cached
        const CACHE_FINAL =  1u32 << 0;
        // Arcs have been cached
        const CACHE_ARCS = 1u32 << 1;
        // Initialized by GC
        const CACHE_INIT = 1u32 << 2;
        // Visited since GC
        const CACHE_RECENT = 1u32 << 3;

        const DEFAULT_CACHE_FLAGS = Self::CACHE_FINAL.bits | Self::CACHE_ARCS.bits | Self::CACHE_INIT.bits | Self::CACHE_RECENT.bits;
    }
}