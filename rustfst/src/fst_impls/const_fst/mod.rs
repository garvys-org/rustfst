pub use self::data_structure::ConstFst;

mod converters;
mod data_structure;
mod expanded_fst;
mod fst;
mod iterators;
mod misc;
mod serializable_fst;

pub(super) static CONST_MIN_FILE_VERSION: i32 = 1;
pub(super) static CONST_ALIGNED_FILE_VERSION: i32 = 1;
pub(super) static CONST_FILE_VERSION: i32 = 2;
pub(super) static CONST_ARCH_ALIGNMENT: usize = 16;
