pub(crate) mod config;
mod replace_fst;
pub(crate) mod replace_fst_op;
mod replace_static;
pub(crate) mod state_table;
pub(crate) mod utils;

pub use replace_fst::ReplaceFst;
pub use replace_static::replace;
